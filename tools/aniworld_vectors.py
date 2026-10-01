"""Hermetic host/Android wire inputs. No provider/network calls, ever."""
import copy, hashlib, json, sys
from html.parser import HTMLParser
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
class StartTagCounter(HTMLParser):
    def __init__(self):super().__init__();self.count=0
    def handle_starttag(self,tag,attrs):self.count+=1
    def handle_startendtag(self,tag,attrs):self.count+=1
def padded_page(body,target_bytes,target_start_tags,missing_wrapper_close=False):
    if missing_wrapper_close:body=body.replace('<body>','<body>\n  <div id="wrapper">',1)
    counter=StartTagCounter();counter.feed(body);needed=target_start_tags-counter.count
    if needed<0:raise ValueError('synthetic base exceeds target tag density')
    noise='<aside><span>ignored synthetic layout padding</span></aside>'*(needed//2)
    if needed%2:noise+='<aside></aside>'
    if '</body>' not in body:raise ValueError('synthetic page needs a body close')
    before,after=body.rsplit('</body>',1)
    fixed=(before+noise+'<!----></body>'+after).encode('utf-8')
    comment_bytes=target_bytes-len(fixed)
    if comment_bytes<0:raise ValueError('synthetic base exceeds target byte size')
    result=before+noise+'<!--'+('x'*comment_bytes)+'--></body>'+after
    if len(result.encode('utf-8'))!=target_bytes:raise AssertionError('synthetic byte sizing')
    verify=StartTagCounter();verify.feed(result)
    if verify.count!=target_start_tags:raise AssertionError('synthetic tag sizing')
    return result
put('release-plan',{'schemaVersion':1,'context':context},count=4)
responses=[response(r,(FIX/({'CALENDAR':'calendar','RECENT':'recent','POSTPONEMENT':'postponement','DIRECT':'episode'}[r]+'.html')).read_text()) for r in roles]
put('release-parse',{'schemaVersion':1,'context':context,'responses':responses},count=7,outcomes=['SUCCESS']*4)
for prefix,role,count in [('calendar','CALENDAR',2),('recent','RECENT',2),('postponement','POSTPONEMENT',2),('direct','DIRECT',1)]:
    release_case(prefix,role,(FIX/(prefix if prefix!='direct' else 'episode')).with_suffix('.html').read_text(),count,'SUCCESS')
for fixture,count,outcome in [('bot',0,'FAILURE'),('truncated',0,'FAILURE'),('malformed-date',0,'PARTIAL'),('duplicate-row',2,'SUCCESS'),('unknown-track',0,'PARTIAL')]:
    release_case(fixture,'RECENT',(FIX/(fixture+'.html')).read_text(),count,outcome)
release_case('challenge','RECENT',(FIX/'bot.html').read_text(),0,'FAILURE')
foreign=(FIX/'recent.html').read_text().replace('Episode 1 mit deutschen Untertiteln','Episode 1 auf Englisch').replace('Deutsche Untertitel Flagge, German Subtitle Flag','Englische Flagge, English Flag')
release_case('known-foreign-track','RECENT',foreign,1,'SUCCESS')
conflicting_recent=(FIX/'recent.html').read_text().replace('title="Episode 1 mit deutschen Untertiteln"','title="Episode 1 mit englischen Untertiteln"')
release_case('conflicting-language-marker-recent','RECENT',conflicting_recent,1,'PARTIAL')
# Calendar malformed rows remain row-local: a bad time keeps the other track's
# valid forecast, while an impossible day makes all rows partial and non-factual.
calendar=(FIX/'calendar.html').read_text()
release_case('calendar-malformed-time-partial','CALENDAR',calendar.replace('~ 12:00 Uhr','~ 25:99 Uhr'),1,'PARTIAL')
release_case('calendar-invalid-day-partial','CALENDAR',calendar.replace('30.09.2026','31.02.2026'),0,'PARTIAL')
episode=(FIX/'episode.html').read_text()
conflicting_episode=episode.replace('title="mit Untertitel Deutsch"','title="mit Untertitel Englisch"')
release_case('conflicting-language-marker-direct','DIRECT',conflicting_episode,0,'FAILURE')
duplicate_foreign=episode.replace('<img data-lang-key="2" title="mit Untertitel Englisch" alt="English subtitles">','<img data-lang-key="2" title="mit Untertitel Englisch" alt="English subtitles"><img data-lang-key="2" title="auf Englisch" alt="English dub">')
release_case('repeated-foreign-data-lang-key-direct','DIRECT',duplicate_foreign,0,'FAILURE')
multiple_foreign=episode.replace('<img data-lang-key="2" title="mit Untertitel Englisch" alt="English subtitles">','<img data-lang-key="2" title="mit Untertitel Englisch" alt="English subtitles"><img data-lang-key="4" title="auf Französisch" alt="French dub">')
release_case('distinct-foreign-data-lang-keys-direct','DIRECT',multiple_foreign,1,'SUCCESS')
captured_german_sub=episode.replace('<img data-lang-key="3" title="mit Untertitel Deutsch" alt="German subtitles">','<img src="/public/img/japanese-german.svg" data-lang-key="3" title="mit Untertitel Deutsch" alt="Ger-Sub, Deutscher Untertitel, Flagge, Sprache">')
release_case('capture-shaped-japanese-german-subtitle-direct','DIRECT',captured_german_sub,1,'SUCCESS')
postponement=(FIX/'postponement.html').read_text()
conflicting_notice_track=postponement.replace('(Sub)','(Sub) (Dub)').replace('(Dub)','(Sub) (Dub)')
release_case('conflicting-postponement-track-markers','POSTPONEMENT',conflicting_notice_track,0,'PARTIAL')
conflicting_notice_direction=postponement.replace('▼','▼ ▲')
release_case('conflicting-postponement-direction-markers','POSTPONEMENT',conflicting_notice_direction,0,'PARTIAL')
release_case('empty','RECENT','',0,'FAILURE')
# A page that advertises a stream but lacks hosterSiteTitle coordinates has no
# structural identity proof, so direct-release parsing must fail closed.
release_case('missing-direct','DIRECT',(FIX/'missing-episode.html').read_text(),0,'FAILURE')
def direct_track_case(name,body,track,count,outcome):
    c=copy.deepcopy(context);c['sourceRoles']=['DIRECT'];c['targets'][0]['track']=track
    put(name+'-release-parse',{'schemaVersion':1,'context':c,'responses':[response('DIRECT',body)]},count=count,outcomes=[outcome])
direct_track_case('direct-missing-track',(FIX/'episode.html').read_text(),'DE_DUB',0,'SUCCESS')
coordinate_mismatch=(FIX/'episode.html').read_text().replace('data-episode="1"','data-episode="2"')
release_case('direct-coordinate-mismatch','DIRECT',coordinate_mismatch,0,'FAILURE')
split_context=copy.deepcopy(context);split_context['sourceRoles']=['DIRECT'];split_context['targets'][0]['sourceSeason']=1;split_context['targets'][0]['navigationSeason']=2;split_context['targets'][0]['installment']['number']='15'
split_body=(FIX/'episode.html').read_text().replace('staffel-1/episode-1','staffel-2/episode-15').replace('data-season="1"','data-season="2"').replace('data-episode="1"','data-episode="15"')
split_url=ORIGIN+'/anime/stream/fixture-series/staffel-2/episode-15'
split_response={'requestId':'direct-0','sourceRole':'DIRECT','status':'OK','httpStatus':200,'finalUrl':split_url,'bodyUtf8':split_body,'sourceHash':hashlib.sha256(split_body.encode()).hexdigest()}
put('split-direct-release-parse',{'schemaVersion':1,'context':split_context,'responses':[split_response]},count=1,outcomes=['SUCCESS'])
bad=response('RECENT',(FIX/'recent.html').read_text());bad['finalUrl']=urls['CALENDAR']
c=copy.deepcopy(context);c['sourceRoles']=['RECENT'];put('redirect-release-parse',{'schemaVersion':1,'context':c,'responses':[bad]},count=0,outcomes=['FAILURE'])
calendar_large=padded_page((FIX/'calendar.html').read_text(),326112,1516)
release_case('calendar-large-dom','CALENDAR',calendar_large,2,'SUCCESS')
recent_row='''
      <div class="col-md-12"><div class="row"><div class="col-md-12">
        <a href="/anime/stream/{key}/staffel-1/episode-1">
          <strong>{title}</strong><span class="listTag bigListTag blue2">S01 E01</span>
          <span class="elementFloatRight">30.09.2026</span>
        </a>
        <img class="flag" title="Episode 1 mit deutschen Untertiteln" alt="Deutsche Untertitel Flagge, German Subtitle Flag">
        <span class="listTag bigListTag green right">Neu!</span>
      </div></div></div>'''
recent_rows=''.join(recent_row.format(key=f'synthetic-series-{n:03d}',title=f'Synthetic Series {n:03d}') for n in range(1,146))
recent_large='''<!doctype html><html lang="de"><head><meta charset="utf-8"><title>Synthetic recent</title></head>
<body><div class="container"><div class="pageTitle pageCenter"><h1>Neue Episoden</h1></div>
<div class="newEpisodeList"><div class="rows">'''+recent_rows+'''</div><div class="cf"></div></div></div></body></html>'''
recent_large=padded_page(recent_large,205679,1367)
release_case('recent-145-rows-large-dom','RECENT',recent_large,145,'SUCCESS')
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
overview_large=padded_page((FIX/'series.html').read_text(),70*1024,620,missing_wrapper_close=True)
nav('overview-large-missing-wrapper-close','OVERVIEW',overview_large,count=1)
episode_large=padded_page(episode,90*1024,620,missing_wrapper_close=True)
nav('episode-large-missing-wrapper-close','EPISODE',episode_large,1,'1','DE_SUB',1)
# Current public captures have denser layout than the original reconstruction.
calendar_current=padded_page((FIX/'calendar.html').read_text(),326112,2012)
release_case('calendar-current-density','CALENDAR',calendar_current,2,'SUCCESS')
recent_current='<!doctype html><html><head><title>Synthetic</title></head><body><h1>Neue Episoden</h1><div class="newEpisodeList">'+''.join(recent_row.format(key=f'synthetic-series-{n:03d}',title=f'Synthetic Series {n:03d}') for n in range(1,151))+'</div></body></html>'
recent_current=padded_page(recent_current,205679,1675)
release_case('recent-150-rows-current-density','RECENT',recent_current,150,'SUCCESS')
release_case('hostile-depth','RECENT','<div>'*65+'</div>'*65,0,'FAILURE')
release_case('hostile-nodes','RECENT','<i/>'*16400,0,'FAILURE')
release_case('hostile-large-truncated','RECENT','<main><!--'+'x'*(128*1024),0,'FAILURE')
release_case('hostile-duplicate-attribute','RECENT', '<h1 class="a" class="b">Neue Episoden</h1>',0,'FAILURE')
release_case('hostile-logical-rows','RECENT','<h1>Neue Episoden</h1><div class="newEpisodeList">'+('<a href="/anime/stream/a/staffel-1/episode-1"><strong>A</strong></a>'*513)+'</div>',0,'FAILURE')
# Reconstruct the current capture's attribute and JSON-escape density, as well as
# bytes/tags/facts. Comment-only padding missed the raw-page fuel regression.
def capture_density_page(body, target_bytes, target_tags, target_attrs, target_crlfs):
    class ShapeCounter(HTMLParser):
        def __init__(self):super().__init__();self.tags=0;self.attrs=0
        def handle_starttag(self,tag,attrs):self.tags+=1;self.attrs+=len(attrs)
        def handle_startendtag(self,tag,attrs):self.handle_starttag(tag,attrs)
    c=ShapeCounter();c.feed(body)
    tags=target_tags-c.tags;attrs=target_attrs-c.attrs
    if tags <= 0 or attrs < 0 or attrs > tags*16:raise ValueError('invalid synthetic density budget')
    noise=''
    for i in range(tags):
        n=attrs//(tags-i);attrs-=n
        noise+='<i'+''.join(f' data-pad-{j}="inert"' for j in range(n))+'>'+'layout'+'</i>\n'
    body=body.replace('</body>',noise+'</body>')
    body=body.replace('\n','\r\n')
    crlfs=target_crlfs-body.count('\r\n')
    if crlfs < 0:raise ValueError('too many synthetic line endings')
    padding='\r\n'*crlfs
    remaining=target_bytes-len(body.encode())-len(padding)-7
    if remaining<0:raise ValueError('synthetic density exceeds byte target')
    body=body.replace('</body>','<!--'+padding+'x'*remaining+'--></body>')
    assert len(body.encode())==target_bytes
    c=ShapeCounter();c.feed(body)
    assert (c.tags,c.attrs,body.count('\r\n'))==(target_tags,target_attrs,target_crlfs)
    return body
calendar_cards=''.join('''<div class="col-md-15 col-sm-3 col-xs-6"><a href="/anime/stream/synthetic-calendar-{n}"><h3 class="seriesTitle">Synthetic Calendar {n}<span class="paragraph-end"></span></h3><small>S01E01<img class="flag" title="Episode 1 mit deutschem Untertitel" alt="Deutscher Untertitel Flagge, German Subtitle Flag"></small><small>~ 12:00 Uhr</small></a></div>'''.format(n=n) for n in range(95))
calendar_capture='<html><head><title>Synthetic capture density</title></head><body><h1>Animekalender</h1><section class="calendarList"><h3>Mittwoch, 30.09.2026</h3><div class="seriesListContainer">'+calendar_cards+'</div></section></body></html>'
calendar_capture=capture_density_page(calendar_capture,326112,2012,3676,2283)
release_case('calendar-95-rows-capture-attribute-density','CALENDAR',calendar_capture,95,'SUCCESS')
recent_capture='<html><head><title>Synthetic capture density</title></head><body><h1>Neue Episoden</h1><div class="newEpisodeList">'+''.join(recent_row.format(key=f'synthetic-series-{n:03d}',title=f'Synthetic Series {n:03d}') for n in range(1,151))+'</div></body></html>'
recent_capture=capture_density_page(recent_capture,205679,1675,2492,2139)
release_case('recent-150-rows-capture-attribute-density','RECENT',recent_capture,150,'SUCCESS')

(OUT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print('Wrote',len(cases),'hermetic provider inputs')
