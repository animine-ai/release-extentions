"""AREX v1 offline tooling. No HTTP client and no production key generation.

Signed schemas contain only safe integers, strings, booleans, arrays and objects.
JCS below deliberately rejects floating point rather than approximating RFC8785.
"""
from __future__ import annotations
import argparse, base64, copy, hashlib, io, json, os, re, stat, struct, sys, zipfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from urllib.parse import urlsplit
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

ROOT=Path(__file__).resolve().parents[1]
SAFE=9007199254740991
LIMITS={'manifest.json':65536,'module.wasm':8388608,'provenance.json':262144,'NOTICE':262144,'package.sig':1024}
ROLES={'CALENDAR','RECENT','POSTPONEMENT','DIRECT'}
NAV={'OVERVIEW_NAVIGATION','EPISODE_NAVIGATION'}
FIXTURE_TIME='2026-09-29T12:00:00Z'
FIXTURE_NOW=datetime.fromisoformat(FIXTURE_TIME.replace('Z','+00:00'))

def fail(message): raise ValueError(message)
def check(ok,message):
    if not ok: fail(message)
def sha(data): return hashlib.sha256(data).hexdigest()
def b64(data): return base64.b64encode(data).decode('ascii')
def unb64(text,size):
    try: data=base64.b64decode(text,validate=True)
    except Exception: fail('invalid base64')
    check(len(data)==size and b64(data)==text,'noncanonical base64')
    return data
def pairs(items):
    out={}
    for k,v in items:
        check(k not in out,'duplicate JSON key');out[k]=v
    return out
def safe_tree(value,depth=0):
    check(depth<=16,'JSON nesting limit')
    if value is None or type(value) is bool:return
    if type(value) is int: check(abs(value)<=SAFE,'unsafe integer');return
    if type(value) is str:
        value.encode('utf-8','strict');return
    if type(value) is list:
        for v in value:safe_tree(v,depth+1)
        return
    if type(value) is dict:
        check(len(value)<=256,'object field limit')
        for k,v in value.items():safe_tree(k,depth+1);safe_tree(v,depth+1)
        return
    fail('floating point or unsupported JSON value')
def strict(data,cap=262144):
    check(0<len(data)<=cap,'JSON byte limit')
    check(not data.startswith(b'\xef\xbb\xbf'),'BOM not allowed')
    try:
        value=json.loads(data.decode('utf-8','strict'),object_pairs_hook=pairs,
                         parse_float=lambda _:fail('floating point unsupported'),parse_constant=lambda _:fail('nonfinite number'))
    except RecursionError:fail('JSON nesting limit')
    safe_tree(value);return value
def jcs(value):
    safe_tree(value)
    def encode(v):
        if type(v) is dict:
            keys=sorted(v,key=lambda s:s.encode('utf-16-be'))
            return '{'+','.join(encode(k)+':'+encode(v[k]) for k in keys)+'}'
        if type(v) is list:return '['+','.join(map(encode,v))+']'
        return json.dumps(v,ensure_ascii=False,separators=(',',':'),allow_nan=False)
    return encode(value).encode('utf-8')
DEFS=json.loads((ROOT/'sdk/schema/v1.schema.json').read_text())['$defs']
def validate(value,name):
    def run(v,s):
        if '$ref' in s:return run(v,DEFS[s['$ref'].split('/')[-1]])
        if 'oneOf' in s:
            hits=0
            for opt in s['oneOf']:
                try:run(v,opt);hits+=1
                except ValueError:pass
            check(hits==1,'schema oneOf');return
        if 'const' in s:check(type(v) is type(s['const']) and v==s['const'],'schema const')
        if 'enum' in s:check(v in s['enum'],'unknown enum')
        t=s.get('type')
        if t=='object':
            check(type(v) is dict,'object required');check(set(v)==set(s['required']),'missing or unknown field')
            for k,w in v.items():run(w,s['properties'][k])
        elif t=='array':
            check(type(v) is list,'array required');check(s.get('minItems',0)<=len(v)<=s.get('maxItems',SAFE),'array bound')
            if s.get('uniqueItems'):check(len({jcs(w) for w in v})==len(v),'duplicate array item')
            for w in v:run(w,s['items'])
        elif t=='string':
            check(type(v) is str,'string required');v.encode('utf-8','strict')
            check(s.get('minLength',0)<=len(v)<=s.get('maxLength',SAFE),'string length')
            # v1 caps are UTF-8 byte caps; JSON Schema maxLength alone is insufficient.
            check(len(v.encode())<=s.get('x-maxUtf8Bytes',s.get('maxLength',SAFE)),'UTF-8 byte bound')
            if 'pattern' in s:check(re.fullmatch(s['pattern'],v) is not None,'string pattern')
        elif t=='integer':check(type(v) is int and s.get('minimum',-SAFE)<=v<=s.get('maximum',SAFE),'integer bound')
        elif t=='boolean':check(type(v) is bool,'boolean required')
        elif t=='null':check(v is None,'null required')
    safe_tree(value);run(value,DEFS[name]);return value
def time(text):
    check(type(text) is str and text.endswith('Z'),'UTC timestamp required')
    return datetime.fromisoformat(text.replace('Z','+00:00'))
def test_key(label):
    # PUBLIC TEST SEEDS. NEVER USE THESE KEYS FOR PRODUCTION.
    return Ed25519PrivateKey.from_private_bytes(hashlib.sha256(('AREX-EP03-PUBLIC-TEST-ONLY:'+str(label)).encode()).digest())
def public(key):return key.public_key().public_bytes(Encoding.Raw,PublicFormat.Raw)
def key_id(key):return sha(public(key))
def signature(signed,key,domain):
    return {'algorithm':'Ed25519','keyId':key_id(key),'signature':b64(key.sign(('AREX-'+domain+'-V1\n').encode()+jcs(signed)))}
def envelope(signed,keys,domain):return {'signed':signed,'signatures':[signature(signed,k,domain) for k in keys]}
def threshold(env,role,keys,revoked,domain):
    sigs=env['signatures'];check(len({s['keyId'] for s in sigs})==len(sigs),'duplicate signature signer')
    valid=0
    for sig in sigs:
        validate(sig,'Signature');raw=unb64(sig['signature'],64);kid=sig['keyId']
        if kid in role['keyIds'] and kid not in revoked:
            try:Ed25519PublicKey.from_public_bytes(unb64(keys[kid],32)).verify(raw,('AREX-'+domain+'-V1\n').encode()+jcs(env['signed']));valid+=1
            except Exception:pass
    check(valid>=role['threshold'],'signature threshold not met')
def verify_root(env,pin,now,previous=None):
    validate(env,'Root');s=env['signed']
    check(s['repositoryId']==pin['repositoryId'],'repository binding')
    check(now<time(s['expiresAt']),'expired root')
    if previous is None:
        check(s['version']==1 and sha(jcs(s))==pin['initialRootSha256'],'initial root pin mismatch')
        check(s['roles']['root']['threshold']==2 and len(s['roles']['root']['keyIds'])==3,'initial 2-of-3 required')
    else:
        p=previous['signed'];check(s['version']==p['version']+1,'nonsequential root rotation')
        check(set(s['revokedKeys'])>=set(p['revokedKeys']) and set(s['revokedDigests'])>=set(p['revokedDigests']),'revocation rollback')
        threshold(env,p['roles']['root'],{k['keyId']:k['publicKey'] for k in p['keys']},p['revokedKeys'],'ROOT')
    keys={k['keyId']:k['publicKey'] for k in s['keys']}
    check(len(keys)==len(s['keys']),'duplicate key')
    for kid,pub in keys.items():check(kid==sha(unb64(pub,32)),'key id mismatch')
    for role in s['roles'].values():
        check(set(role['keyIds'])<=set(keys) and 1<=role['threshold']<=len(role['keyIds']),'invalid role')
        check(not(set(role['keyIds'])&set(s['revokedKeys'])),'revoked role key')
    for p in s['publishers']:
        check(p['keyId'] in keys and p['keyId'] not in s['revokedKeys'],'publisher key revoked or absent')
        check(time(p['notBefore'])<time(p['expiresAt']),'publisher interval')
    threshold(env,s['roles']['root'],keys,s['revokedKeys'],'ROOT');return env
def verify_index(env,root,pin,now,last=None):
    validate(env,'Index');s=env['signed'];r=root['signed']
    check(now<time(r['expiresAt']) and s['repositoryId']==r['repositoryId'] and s['rootVersion']==r['version'],'index root binding')
    issued,expiry=time(s['issuedAt']),time(s['expiresAt'])
    check(issued<=now+timedelta(seconds=600) and now<expiry and issued<expiry<=issued+timedelta(days=7),'index expiry/issuedAt')
    if last:
        old=last['signed'];check(s['sequence']>old['sequence'] or s['sequence']==old['sequence'] and jcs(s)==jcs(old),'index replay/equivocation')
    threshold(env,r['roles']['index'],{k['keyId']:k['publicKey'] for k in r['keys']},r['revokedKeys'],'INDEX')
    order=[]
    for e in s['entries']:
        u=urlsplit(e['packageUrl']);check(u.scheme=='https' and u.hostname and not u.username and not u.password and u.port is None and not u.query and not u.fragment,'package URL policy')
        check('https://'+u.hostname in pin['distributionOrigins'] and '%' not in u.path and '..' not in u.path and u.path.endswith('/'+e['archiveSha256']+'.arex'),'immutable URL binding')
        check(any(p['publisherId']==e['publisherId'] and p['extensionId']==e['extensionId'] and p['providerId']==e['providerId'] and p['keyId']==e['keyId'] and time(p['notBefore'])<=now<time(p['expiresAt']) for p in r['publishers']),'publisher scope missing')
        order.append((e['extensionId'],e['releaseSequence']))
    check(all(a<b for a,b in zip(order,order[1:])),'catalog order/duplicate release');return env
def read_archive(data):
    check(0<len(data)<=8388608,'archive bound')
    check(len(data)>=22 and data[-22:-18]==b'PK\x05\x06','canonical ZIP end record')
    end=struct.unpack_from('<4s4H2IH',data,len(data)-22)
    _,disk,central_disk,n_disk,n,size,offset,comment=end
    check(disk==central_disk==0 and n_disk==n==5 and comment==0 and offset+size==len(data)-22,'central directory/end binding')
    out={}
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        check(not z.comment,'archive comment')
        entries=z.infolist();check(len(entries)==5 and {e.filename for e in entries}==set(LIMITS),'root-only archive entries')
        check(sum(e.file_size for e in entries)<=12582912,'uncompressed archive bound')
        for e in entries:
            check(e.filename not in out and 0<e.file_size<=LIMITS[e.filename],'entry duplicate/size')
            check(e.compress_type in [zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED] and not e.flag_bits&1,'unsupported archive encoding')
            check(not e.extra and not e.comment and e.create_system in [0,3],'extra ZIP metadata')
            mode=e.external_attr>>16;check(stat.S_IFMT(mode) in [0,stat.S_IFREG],'symlink/special file')
            out[e.filename]=z.read(e)
        ordered=sorted(entries,key=lambda e:e.header_offset);check(ordered[0].header_offset==0,'ZIP prefix')
        for i,e in enumerate(ordered):
            start=e.header_offset;check(start+30<=offset,'local header bound')
            h=struct.unpack_from('<4s5H3I2H',data,start)
            marker,version,flags,method,_,_,crc,compressed,uncompressed,name_len,extra_len=h
            check(marker==b'PK\x03\x04' and flags==e.flag_bits and method==e.compress_type and flags&~0x808==0,'local header')
            check(extra_len==0 and data[start+30:start+30+name_len]==e.filename.encode('ascii'),'local name/extra')
            finish=start+30+name_len+extra_len+e.compress_size
            boundary=ordered[i+1].header_offset if i+1<len(ordered) else offset
            if flags&8:
                descriptor=data[finish:boundary]
                if len(descriptor)==16:check(descriptor[:4]==b'PK\x07\x08','descriptor signature');descriptor=descriptor[4:]
                check(len(descriptor)==12,'descriptor bound')
                check(struct.unpack('<3I',descriptor)==(e.CRC,e.compress_size,e.file_size),'descriptor binding')
                check(crc in [0,e.CRC] and compressed in [0,e.compress_size] and uncompressed in [0,e.file_size],'local descriptor mirrors')
            else:check(finish==boundary and (crc,compressed,uncompressed)==(e.CRC,e.compress_size,e.file_size),'local/central boundary mismatch')
    # Host's raw central/local-header reader is exercised in compatibility CI.
    return out
def verify_package(data,entry,root,now):
    check(not entry['yanked'] and not entry['revoked'],'yanked/revoked package')
    check(len(data)==entry['archiveBytes'] and sha(data)==entry['archiveSha256'],'archive binding')
    r=root['signed'];check(entry['archiveSha256'] not in r['revokedDigests'] and entry['keyId'] not in r['revokedKeys'],'root revocation')
    files=read_archive(data);m=validate(strict(files['manifest.json'],65536),'Manifest');p=validate(strict(files['provenance.json']),'Provenance')
    check(sha(jcs(m))==entry['manifestSha256'],'manifest catalog digest')
    for n in ['extensionId','providerId','displayName','navigationCapabilities','publisherId','keyId','version','releaseSequence']:
        check(m[n]==entry[n],'manifest catalog identity')
    check(1<=len(m['displayName'])<=64 and not any(ord(c)<32 or 0x7f<=ord(c)<=0x9f or 0x202a<=ord(c)<=0x202e or 0x2066<=ord(c)<=0x2069 for c in m['displayName']),'unsafe display name')
    check(m['capabilities'] or m['navigationCapabilities'],'empty capabilities')
    check(not any(h.replace('.','').isdigit() for h in m['allowedHosts']),'IP literal host')
    u=urlsplit(m['sourceRepository']);check(u.scheme=='https' and u.hostname and not u.username and not u.password and not u.query and not u.fragment,'source repository URL')
    scopes=[s for s in r['publishers'] if all(s[k]==m[k] for k in ['publisherId','extensionId','providerId','keyId']) and time(s['notBefore'])<=now<time(s['expiresAt'])]
    check(len(scopes)==1,'publisher scope');s=scopes[0]
    check(set(m['capabilities'])<=set(s['roles']) and set(m['navigationCapabilities'])<=set(s['navigation']) and set(m['allowedHosts'])<=set(s['hosts']),'scope escalation')
    sig=validate(strict(files['package.sig'],1024),'Signature');check(sig['keyId']==m['keyId'],'package signer binding')
    pub=next(k['publicKey'] for k in r['keys'] if k['keyId']==sig['keyId'])
    Ed25519PublicKey.from_public_bytes(unb64(pub,32)).verify(unb64(sig['signature'],64),b'AREX-PACKAGE-V1\n'+jcs(m))
    for n,f in [('module','module.wasm'),('provenance','provenance.json'),('notice','NOTICE')]:
        check(m['digests'][n]=={'sha256':sha(files[f]),'bytes':len(files[f])},'content digest/size')
    for a,b in [('sourceRepository','sourceRepository'),('sourceCommit','sourceCommit')]:check(p[a]==m[b],'provenance source binding')
    check(p['moduleDigest']==m['digests']['module']['sha256'] and p['dependencyLockDigest']==m['build']['lockfileDigest'] and p['workflowIdentity']==m['build']['workflowIdentity'],'provenance build binding')
    return m
def build_package(module,source_repo,source_commit,key,lock,workflow='ep03-ci/v1'):
    check(module.startswith(b'\x00asm\x01\x00\x00\x00'),'core wasm required')
    notice=b'AREX fixture and SDK: GPL-3.0-only. Test-only fixture. Not AniWorld.\n'
    p={'schemaVersion':1,'sourceRepository':source_repo,'sourceCommit':source_commit,'licenseSpdx':['GPL-3.0-only'],'components':[{'name':'arex-sdk','origin':source_repo,'path':'sdk/rust','licenseSpdx':'GPL-3.0-only'}],'localModifications':[],'compilerVersion':'rustc 1.95.0 + Binaryen 133','sdkVersion':'0.1.0','dependencyLockDigest':sha(lock),'reproducibleBuildCommand':'bash tools/build_fixture.sh','workflowIdentity':workflow,'moduleDigest':sha(module)}
    prov=jcs(validate(p,'Provenance'))
    m={'schemaVersion':1,'extensionId':'fixture.release','providerId':'fixture','displayName':'Fixture Provider','version':'0.1.0','releaseSequence':1,'hostApiMin':1,'hostApiMax':1,'capabilities':['CALENDAR'],'navigationCapabilities':['OVERVIEW_NAVIGATION','EPISODE_NAVIGATION'],'allowedHosts':['example.org'],'digests':{n:{'sha256':sha(d),'bytes':len(d)} for n,d in [('module',module),('provenance',prov),('notice',notice)]},'publisherId':'fixture.publisher','keyId':key_id(key),'sourceRepository':source_repo,'sourceCommit':source_commit,'build':{'toolchainVersion':'1.95.0','target':'wasm32v1-none','lockfileDigest':sha(lock),'workflowIdentity':workflow}}
    return assemble_package(module,m,p,notice,key)
def assemble_package(module,manifest,provenance,notice,key):
    m=copy.deepcopy(manifest);p=copy.deepcopy(provenance)
    m['keyId']=key_id(key);p['moduleDigest']=sha(module)
    validate(p,'Provenance');prov=jcs(p)
    m['digests']={n:{'sha256':sha(d),'bytes':len(d)} for n,d in [('module',module),('provenance',prov),('notice',notice)]}
    validate(m,'Manifest')
    check(module.startswith(b'\x00asm\x01\x00\x00\x00'),'core wasm required')
    check(p['sourceRepository']==m['sourceRepository'] and p['sourceCommit']==m['sourceCommit'] and p['workflowIdentity']==m['build']['workflowIdentity'] and p['dependencyLockDigest']==m['build']['lockfileDigest'],'builder provenance binding')
    files={'manifest.json':jcs(m),'module.wasm':module,'provenance.json':prov,'NOTICE':notice,'package.sig':jcs(signature(m,key,'PACKAGE'))}
    out=io.BytesIO()
    with zipfile.ZipFile(out,'w',compression=zipfile.ZIP_STORED,allowZip64=False) as z:
        for name,data in files.items():
            check(0<len(data)<=LIMITS[name],'build entry bound')
            entry=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));entry.create_system=3;entry.external_attr=0o100644<<16
            z.writestr(entry,data)
    data=out.getvalue();check(len(data)<=8388608,'built archive bound');return data,m
def fixture_chain(data,manifest):
    keys=[test_key(i) for i in range(5)];ids=[key_id(k) for k in keys]
    signed={'schemaVersion':1,'repositoryId':'fixture.repository','version':1,'expiresAt':'2030-01-01T00:00:00Z','keys':[{'keyId':key_id(k),'publicKey':b64(public(k))} for k in keys],'roles':{'root':{'threshold':2,'keyIds':ids[:3]},'index':{'threshold':1,'keyIds':[ids[3]]}},'publishers':[{'publisherId':manifest['publisherId'],'extensionId':manifest['extensionId'],'providerId':manifest['providerId'],'keyId':ids[4],'roles':manifest['capabilities'],'navigation':manifest['navigationCapabilities'],'hosts':manifest['allowedHosts'],'notBefore':'2026-01-01T00:00:00Z','expiresAt':'2030-01-01T00:00:00Z'}],'revokedKeys':[],'revokedDigests':[]}
    root=envelope(signed,keys[:2],'ROOT')
    pin={'repositoryId':'fixture.repository','initialRootSha256':sha(jcs(signed)),'distributionOrigins':['https://packages.example.org']}
    entry={n:manifest[n] for n in ['extensionId','providerId','displayName','navigationCapabilities','publisherId','keyId','version','releaseSequence','hostApiMin','hostApiMax']}
    entry.update({'packageUrl':'https://packages.example.org/dist/fixture.release/0.1.0/'+sha(data)+'.arex','archiveSha256':sha(data),'archiveBytes':len(data),'manifestSha256':sha(jcs(manifest)),'yanked':False,'revoked':False})
    index=envelope({'schemaVersion':1,'repositoryId':pin['repositoryId'],'rootVersion':1,'sequence':1,'issuedAt':FIXTURE_TIME,'expiresAt':'2026-10-05T12:00:00Z','entries':[entry]},[keys[3]],'INDEX')
    return root,index,pin
def write_chain(output,module,source_repo,commit):
    output.mkdir(parents=True,exist_ok=True)
    data,m=build_package(module,source_repo,commit,test_key(4),(ROOT/'Cargo.lock').read_bytes())
    root,index,pin=fixture_chain(data,m)
    verify_root(root,pin,FIXTURE_NOW);verify_index(index,root,pin,FIXTURE_NOW);verify_package(data,index['signed']['entries'][0],root,FIXTURE_NOW)
    (output/'fixture.arex').write_bytes(data)
    for name,val in [('root.json',root),('index.json',index),('test-pin.json',pin),('manifest.json',m)]: (output/name).write_bytes(jcs(val))
    (output/'SHA256SUMS').write_text(''.join(sha(p.read_bytes())+'  '+p.name+'\n' for p in sorted(output.iterdir()) if p.is_file() and p.name!='SHA256SUMS'))
def cli():
    parser=argparse.ArgumentParser();sub=parser.add_subparsers(dest='cmd',required=True)
    p=sub.add_parser('test-chain');p.add_argument('--module',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--source-repository',required=True);p.add_argument('--source-commit',required=True)
    p=sub.add_parser('verify-chain');p.add_argument('directory',type=Path);p.add_argument('--now',default=FIXTURE_TIME)
    p=sub.add_parser('canonicalize');p.add_argument('input',type=Path)
    p=sub.add_parser('sign-envelope');p.add_argument('--domain',choices=['ROOT','INDEX'],required=True);p.add_argument('--key-file',type=Path,required=True);p.add_argument('input',type=Path);p.add_argument('output',type=Path)
    p=sub.add_parser('build');
    for name in ['module','manifest','provenance','notice','key-file','output']:p.add_argument('--'+name,type=Path,required=True)
    p=sub.add_parser('verify-package')
    for name in ['archive','root','index','pin']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--now',default=datetime.now(timezone.utc).isoformat().replace('+00:00','Z'))
    p=sub.add_parser('verify-root')
    for name in ['root','pin']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--previous',type=Path);p.add_argument('--now',default=datetime.now(timezone.utc).isoformat().replace('+00:00','Z'))
    p=sub.add_parser('prepare-envelope');p.add_argument('--domain',choices=['ROOT','INDEX'],required=True);p.add_argument('input',type=Path);p.add_argument('output',type=Path)
    a=parser.parse_args()
    if a.cmd=='test-chain':write_chain(a.output,a.module.read_bytes(),a.source_repository,a.source_commit)
    elif a.cmd=='verify-chain':
        d=a.directory;root=strict((d/'root.json').read_bytes(),65536);index=strict((d/'index.json').read_bytes());pin=strict((d/'test-pin.json').read_bytes());now=time(a.now)
        verify_root(root,pin,now);verify_index(index,root,pin,now);verify_package((d/'fixture.arex').read_bytes(),index['signed']['entries'][0],root,now);print('TEST chain verified; not a production trust root')
    elif a.cmd=='canonicalize':sys.stdout.buffer.write(jcs(strict(a.input.read_bytes())))
    elif a.cmd=='sign-envelope':
        # Caller supplies an externally protected ephemeral seed file. No key output/logging.
        key=Ed25519PrivateKey.from_private_bytes(a.key_file.read_bytes());env=strict(a.input.read_bytes());validate(env['signed'],a.domain.title()+'Signed')
        env['signatures']=[s for s in env['signatures'] if s['keyId']!=key_id(key)]+[signature(env['signed'],key,a.domain)]
        a.output.write_bytes(jcs(env))
    elif a.cmd=='build':
        key=Ed25519PrivateKey.from_private_bytes(a.key_file.read_bytes())
        data,_=assemble_package(a.module.read_bytes(),strict(a.manifest.read_bytes(),65536),strict(a.provenance.read_bytes()),a.notice.read_bytes(),key);a.output.write_bytes(data)
    elif a.cmd=='verify-package':
        root=strict(a.root.read_bytes(),65536);index=strict(a.index.read_bytes());pin=strict(a.pin.read_bytes());now=time(a.now)
        verify_root(root,pin,now);verify_index(index,root,pin,now);data=a.archive.read_bytes()
        entries=[e for e in index['signed']['entries'] if e['archiveSha256']==sha(data)];check(len(entries)==1,'archive not uniquely indexed');verify_package(data,entries[0],root,now);print('Package cryptographic/container verification passed; native profile check is additionally required')
    elif a.cmd=='verify-root':
        env=strict(a.root.read_bytes(),65536);pin=strict(a.pin.read_bytes());previous=strict(a.previous.read_bytes(),65536) if a.previous else None
        if previous:
            check(previous['signed']['version']==1,'CLI previous root must be the pinned initial root; longer chains require authenticated persistent state')
            verify_root(previous,pin,time(a.now))
        verify_root(env,pin,time(a.now),previous);print('Root verified')
    elif a.cmd=='prepare-envelope':
        signed=strict(a.input.read_bytes());validate(signed,a.domain.title()+'Signed');a.output.write_bytes(jcs({'signed':signed,'signatures':[]}))
if __name__=='__main__':cli()
