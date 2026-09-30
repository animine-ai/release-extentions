"""Hermetic host/Android wire inputs. No provider/network calls, ever."""
import copy, hashlib, json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=Path(sys.argv[1] if len(sys.argv)>1 else ROOT/'build/aniworld-inputs')
OUT.mkdir(parents=True,exist_ok=True)
FIX=ROOT/'sources/aniworld/fixtures'
ORIGIN='https://aniworld.to'
NOW='2026-09-30T12:00:00Z'
roles=['CALENDAR','RECENT','POSTPONEMENT','DIRECT']
urls={'CALENDAR':ORIGIN+'/animekalender','RECENT':ORIGIN+'/neue-episoden','POSTPONEMENT':ORIGIN+'/support/frage/anime-verschiebungen','DIRECT':ORIGIN+'/anime/stream/fixture-series/staffel-1/episode-1'}
ids={'CALENDAR':'calendar','RECENT':'recent','POSTPONEMENT':'postponement','DIRECT':'direct-0'}
target={'targetToken':'t1','providerSeriesKey':'fixture-series','providerUrl':None,'sourceSeason':1,'navigationSeason':1,'installment':{'kind':'EPISODE','number':'1'},'track':'DE_SUB'}
context={'extensionId':'de.aniworld','providerId':'aniworld','sourceRoles':roles,'observedAt':NOW,'targets':[target]}
cases=[]
def put(name,value,**expected):
    (OUT/(name+'-input.json')).write_text(json.dumps(value,ensure_ascii=False,separators=(',',':'))+'\n')
    cases.append({'name':name,**expected})
def response(role,body):
    return {'requestId':ids[role],'sourceRole':role,'status':'OK','httpStatus':200,'finalUrl':urls[role],'bodyUtf8':body,'sourceHash':hashlib.sha256(body.encode()).hexdigest()}
def release_case(prefix,role,body,count,outcome,unknown=False):
    c=copy.deepcopy(context);c['sourceRoles']=[role]
    put(prefix+'-release-parse',{'schemaVersion':1,'context':c,'responses':[response(role,body)]},count=count,outcomes=[outcome],unknown=unknown)
put('release-plan',{'schemaVersion':1,'context':context},count=4)
responses=[response(r,(FIX/({'CALENDAR':'calendar','RECENT':'recent','POSTPONEMENT':'postponement','DIRECT':'episode'}[r]+'.html')).read_text()) for r in roles]
put('release-parse',{'schemaVersion':1,'context':context,'responses':responses},count=7,outcomes=['SUCCESS']*4)
for prefix,role,count in [('calendar','CALENDAR',2),('recent','RECENT',2),('postponement','POSTPONEMENT',2),('direct','DIRECT',1)]:
    release_case(prefix,role,(FIX/(prefix if prefix!='direct' else 'episode')).with_suffix('.html').read_text(),count,'SUCCESS')
for fixture,count,outcome in [('bot',0,'FAILURE'),('truncated',0,'FAILURE'),('malformed-date',0,'PARTIAL'),('duplicate-row',2,'SUCCESS'),('unknown-track',1,'SUCCESS')]:
    release_case(fixture,'RECENT',(FIX/(fixture+'.html')).read_text(),count,outcome,fixture=='unknown-track')
release_case('empty','RECENT','',0,'FAILURE')
# A page that advertises a stream but lacks hosterSiteTitle coordinates has no
# structural identity proof, so direct-release parsing must fail closed.
release_case('missing-direct','DIRECT',(FIX/'missing-episode.html').read_text(),0,'FAILURE')
bad=response('RECENT',(FIX/'recent.html').read_text());bad['finalUrl']=urls['CALENDAR']
c=copy.deepcopy(context);c['sourceRoles']=['RECENT'];put('redirect-release-parse',{'schemaVersion':1,'context':c,'responses':[bad]},count=0,outcomes=['FAILURE'])
def nav(prefix,kind,body,season=None,number=None,track=None,count=1):
    c={'schemaVersion':1,'extensionId':'de.aniworld','providerId':'aniworld','observedAt':NOW,'targetKind':kind,'targetToken':'n1','providerSeriesKey':'fixture-series','providerRouteHint':None,'sourceSeason':season,'providerEpisode':number,'track':track}
    url=ORIGIN+'/anime/stream/fixture-series'+('' if season is None else '/staffel-'+str(season))+('' if number is None else '/episode-'+number)
    put(prefix+'-plan',c,count=1)
    r={'requestId':'navigation','status':'OK','httpStatus':200,'finalUrl':url,'bodyUtf8':body,'sourceHash':hashlib.sha256(body.encode()).hexdigest()}
    put(prefix+'-parse',{'schemaVersion':1,'context':c,'responses':[r]},count=count)
nav('overview','OVERVIEW',(FIX/'series.html').read_text())
episode=(FIX/'episode.html').read_text()
nav('episode','EPISODE',episode,1,'1','DE_SUB')
nav('missing-episode','EPISODE',(FIX/'missing-episode.html').read_text(),1,'1','DE_SUB',0)
nav('unavailable-dub-episode','EPISODE',episode,1,'1','DE_DUB',0)
nav('split-episode','EPISODE',episode.replace('staffel-1/episode-1','staffel-2/episode-15').replace('data-season="1"','data-season="2"').replace('data-episode="1"','data-episode="15"'),2,'15','DE_SUB')
nav('canonical-mismatch-episode','EPISODE',episode.replace('href="https://aniworld.to/anime/stream/fixture-series','href="https://aniworld.to/anime/stream/other'),1,'1','DE_SUB',0)
(OUT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print('Wrote',len(cases),'hermetic provider inputs')
