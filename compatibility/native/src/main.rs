use anyhow::{Result,ensure};
use std::{env,fs,path::Path};
use wasmtime::{Caller,Engine,Linker,Module,Store};
mod host_engine;
fn run(engine:&Engine,module:&Module,export:&str,input:&[u8])->Result<Vec<u8>>{
    let fuel=if export.starts_with("parse_"){20_000_000}else{10_000_000};
    let mut store=Store::new(engine,(0usize,0usize));store.set_fuel(fuel)?;store.set_epoch_deadline(1);
    let mut linker=Linker::new(engine);
    linker.func_wrap("arex_v1","diagnostic",|mut caller:Caller<'_,(usize,usize)>,p:i32,n:i32|->wasmtime::Result<i32>{
      let result=(||->Result<i32>{
        ensure!(p>0 && (0..=256).contains(&n),"diagnostic pointer/size");let memory=caller.get_export("memory").and_then(|e|e.into_memory()).ok_or_else(||anyhow::anyhow!("memory"))?;
        let (count,bytes)=*caller.data();ensure!(count<32 && bytes+n as usize<=8192,"diagnostic budget");
        let start=p as u32 as usize;let end=start.checked_add(n as usize).ok_or_else(||anyhow::anyhow!("overflow"))?;ensure!(end<=memory.data_size(&caller),"diagnostic range");
        std::str::from_utf8(&memory.data(&caller)[start..end])?;*caller.data_mut()=(count+1,bytes+n as usize);Ok(0)
      })();result.map_err(|e|wasmtime::Error::msg(e.to_string()))
    })?;
    let instance=linker.instantiate(&mut store,module)?;let memory=instance.get_memory(&mut store,"memory").unwrap();
    let alloc=instance.get_typed_func::<i32,i32>(&mut store,"arex_alloc")?;let free=instance.get_typed_func::<(i32,i32),()>(&mut store,"arex_free")?;
    let ptr=alloc.call(&mut store,input.len() as i32)?;ensure!(ptr!=0,"allocation");
    ensure!(memory.data(&store).get(ptr as u32 as usize..ptr as u32 as usize+input.len()).is_some_and(|b|b.iter().all(|v|*v==0)),"fresh allocation was not zero initialized");
    memory.write(&mut store,ptr as u32 as usize,input)?;
    let call=instance.get_typed_func::<(i32,i32),i64>(&mut store,export)?;let packed=call.call(&mut store,(ptr,input.len() as i32))? as u64;
    let p=(packed>>32) as u32 as usize;let len=packed as u32 as usize;ensure!(p>0 && len>0 && len<=1024*1024 && p.checked_add(len).is_some_and(|e|e<=memory.data_size(&store)),"output bounds");
    ensure!(p+len<=ptr as u32 as usize || p>=ptr as u32 as usize+input.len(),"overlapping input/output");
    let mut output=vec![0u8;len];memory.read(&store,p,&mut output)?;free.call(&mut store,(p as i32,len as i32))?;free.call(&mut store,(ptr,input.len() as i32))?;Ok(output)
}
fn main()->Result<()> {
    let args:Vec<_>=env::args().collect();let module=fs::read(&args[1])?;let input=Path::new(&args[2]);let out=Path::new(&args[3]);fs::create_dir_all(out)?;
    let engine=host_engine::build_engine()?;Module::validate(&engine,&module)?;let compiled=Module::new(&engine,&module)?;
    let mut simd=b"\0asm\x01\0\0\0".to_vec();simd.extend_from_slice(&[1,4,1,0x60,0,0,3,2,1,0,10,23,1,21,0,0xfd,0x0c]);simd.extend_from_slice(&[0;16]);simd.extend_from_slice(&[0x1a,0x0b]);
    ensure!(Module::validate(&engine,&simd).is_err(),"forbidden SIMD accepted");
    let mut invalid=b"\0asm\x01\0\0\0".to_vec();invalid.extend_from_slice(&[1,4,1,0x60,0,0,3,2,1,0,10,5,1,3,0,0xff,0x0b]);
    ensure!(Module::validate(&engine,&invalid).is_err(),"invalid opcode accepted");
    let mut inputs:Vec<_>=fs::read_dir(input)?.filter_map(|e|e.ok()).map(|e|e.path()).filter(|p|p.file_name().is_some_and(|n|n.to_string_lossy().ends_with("-input.json"))).collect();inputs.sort();
    let mut metrics=Vec::new();
    for path in inputs {
        let file=path.file_name().unwrap().to_string_lossy();let name=file.strip_suffix("-input.json").unwrap();
        let export=if name.contains("release-plan"){"plan_requests"}else if name.contains("release-parse"){"parse_responses"}else if name.ends_with("-plan"){"plan_navigation"}else{"parse_navigation"};
        let bytes=fs::read(&path)?;
        let first=run(&engine,&compiled,export,&bytes)?;let second=run(&engine,&compiled,export,&bytes)?;ensure!(first==second,"nondeterministic fixture output");fs::write(out.join(format!("{name}-output.json")),&first)?;
        let mut samples=Vec::new();for _ in 0..50 {let start=std::time::Instant::now();run(&engine,&compiled,export,&bytes)?;samples.push(start.elapsed().as_micros());}samples.sort();
        metrics.push(format!("{{\"case\":\"{name}\",\"inputBytes\":{},\"outputBytes\":{},\"p50Micros\":{},\"p95Micros\":{},\"samples\":50,\"memoryBytes\":33554432,\"fuelCeiling\":{}}}",bytes.len(),first.len(),samples[25],samples[47],if export.starts_with("parse_"){20_000_000}else{10_000_000}));
    }
    fs::write(out.join("performance.json"),format!("[{}]",metrics.join(",")))?;

    println!("Wasmtime 48.0.3 accepted host feature profile + ABI execution + negative SIMD/opcode vectors passed");Ok(())
}
