"""Deterministic public test vectors shared by Python and the actual host verifier."""
import copy,sys
from pathlib import Path
base=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(base/'tools'),str(base/'tests')]
import arex as a
from test_tools import rotation
def generate(directory):
    root=a.strict((directory/'root.json').read_bytes());index=a.strict((directory/'index.json').read_bytes())
    rot=rotation(root);no_old=copy.deepcopy(rot);no_old['signatures']=rot['signatures'][2:]
    no_new=copy.deepcopy(rot);no_new['signatures']=rot['signatures'][:2]
    threshold=copy.deepcopy(root);threshold['signatures']=threshold['signatures'][:1]
    duplicate=copy.deepcopy(root);duplicate['signatures']=[duplicate['signatures'][0]]*2
    vectors=[{'name':n,'kind':'root','previous':prev,'valid':valid,'envelope':env} for n,prev,valid,env in [('root-ok',False,True,root),('root-threshold',False,False,threshold),('root-duplicate',False,False,duplicate),('rotation-dual',True,True,rot),('rotation-missing-old',True,False,no_old),('rotation-missing-new',True,False,no_new)]]
    for name,mut,valid in [('index-ok',{},True),('index-next',{'sequence':2},True),('index-equivocation',{'issuedAt':'2026-09-29T11:59:00Z'},False),('index-expired',{'expiresAt':'2026-09-28T00:00:00Z'},False),('index-future',{'issuedAt':'2026-09-30T12:00:00Z'},False),('index-too-long',{'expiresAt':'2030-01-01T00:00:00Z'},False)]:
        signed=copy.deepcopy(index['signed']);signed.update(mut);env=a.envelope(signed,[a.test_key(3)],'INDEX')
        vectors.append({'name':name,'kind':'index','previous':True,'valid':valid,'envelope':env})
    for name,field in [('index-yank','yanked'),('index-revocation','revoked')]:
        s=copy.deepcopy(index['signed']);s['sequence']=2;s['entries'][0][field]=True
        vectors.append({'name':name,'kind':'blocked-package','previous':True,'valid':False,'envelope':a.envelope(s,[a.test_key(3)],'INDEX')})
    (directory/'trust-vectors.json').write_bytes(a.jcs(vectors))
if __name__=='__main__':generate(Path(sys.argv[1]))
