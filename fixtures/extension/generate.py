import sys
from pathlib import Path
base=Path(__file__).resolve().parents[2];sys.path.insert(0,str(base/'tools'))
import arex as a
def generate():
    output=base/'fixtures/wire';output.mkdir(parents=True,exist_ok=True)
    context={'extensionId':'fixture.release','providerId':'fixture','sourceRoles':['CALENDAR'],'observedAt':a.FIXTURE_TIME,'targets':[]}
    response={'requestId':'calendar-1','sourceRole':'CALENDAR','status':'OK','httpStatus':200,'finalUrl':'https://example.org/calendar','bodyUtf8':'fixture-release-v1','sourceHash':a.sha(b'fixture-release-v1')}
    items={'release-plan':{'schemaVersion':1,'context':context},'release-parse':{'schemaVersion':1,'context':context,'responses':[response]}}
    for kind in ['OVERVIEW','EPISODE']:
        c={'schemaVersion':1,'extensionId':'fixture.release','providerId':'fixture','observedAt':a.FIXTURE_TIME,'targetKind':kind,'targetToken':'t1','providerSeriesKey':'series-1','providerRouteHint':None,'sourceSeason':2,'providerEpisode':'15' if kind=='EPISODE' else None,'track':'DE_SUB' if kind=='EPISODE' else None}
        r={'requestId':'nav-1','status':'OK','httpStatus':200,'finalUrl':'https://example.org/nav-source','bodyUtf8':'fixture-nav-v1','sourceHash':a.sha(b'fixture-nav-v1')}
        items[kind.lower()+'-plan']=c;items[kind.lower()+'-parse']={'schemaVersion':1,'context':c,'responses':[r]}
    for name,v in items.items():(output/(name+'-input.json')).write_bytes(a.jcs(v))
if __name__=='__main__':generate()
