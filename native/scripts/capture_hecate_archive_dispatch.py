"""Trace the installed Hecate journal dispatcher's original flattened branches.

Only original arithmetic, CLR type tests, casts, and declared synthetic input
properties are evaluated. No installed code or network/I/O side effect runs.
"""
from pathlib import Path
import re,json,sys
source=Path('/home/loic/.cache/observatory-readable/hecate/Hecate/HecateWorker.cs').read_text()
a=source.index('switch ((num3 = (uint)(num6 ^ 0x190D1255)) % 86)');b=source.index('IL_0eab:',a)
body=source[a:b];starts=list(re.finditer(r'case (\d+)u:',body[:body.index('IL_0dff:')]))
cases={int(m[1]):body[m.end():starts[i+1].start() if i+1<len(starts) else body.index('IL_0dff:')] for i,m in enumerate(starts)}
label_starts=list(re.finditer(r'\b(IL_[0-9a-f]+):',body))
labels={m[1]:body[m.end():label_starts[i+1].start() if i+1<len(label_starts) else len(body)] for i,m in enumerate(label_starts)}
parents=json.loads((Path(__file__).resolve().parents[1]/'fixtures/explorer-dto-schema.json').read_text())['Types']
getters={m[1]:m[2] for m in re.finditer(r'static [^\n]+ (\w+)\([^\n]*\)\s*\{\s*return P_0\.(\w+);\s*\}',source)}
class OpaqueCondition(Exception):pass
def bracket(s,i,left='(',right=')'):
 n=1;j=i+1
 while n:n+=(s[j]==left)-(s[j]==right);j+=1
 return j
def trace(event,props=None):
 props=props or {};v={'num6':113768627};live={};steps=[];labels_seen=[]
 def is_type(name):
  current=event
  while current:
   if current==name:return True
   current=parents.get(current,{}).get('Base')
  return False
 def expr(x):
  x=re.sub(r'\((?:int|uint)\)','',x)
  return eval(x,{'__builtins__':{}},v)&0xffffffff
 def cond(x):
  if x in props.get('__predicates',{}):return props['__predicates'][x]
  if 'journal is' in x:
   found=is_type(re.search(r'journal is (\w+)',x)[1]);return not found if x.strip().startswith('!') else found
  m=re.fullmatch(r'(val\d*|_003Clocation_003E5__2) (==|!=) null',x)
  if m:return not live.get(m[1],False) if m[2]=='==' else live.get(m[1],False)
  if x=='awaiter.IsCompleted':return True
  if '== hecateWorker.currentID64' in x:return props.get('SameSystem',False)
  if '!= hecateWorker.currentID64' in x:return not props.get('SameSystem',False)
  numeric=re.fullmatch(r'(\w+)\(.*\) (==|!=) 0',x)
  if numeric and numeric[1] in getters:
   value=props.get(getters[numeric[1]],1)
   return value==0 if numeric[2]=='==' else value!=0
  helper=re.search(r'(_\w+)\((val\d*)\)',x)
  if helper and helper[1] in getters:
   name=getters[helper[1]]
   if name in props:return not props[name] if x.startswith('!') else props[name]
  raise OpaqueCondition(x)
 pending=None
 for step in range(250):
  if pending is None:
   v['num3']=v['num6']^0x190D1255;case=v['num3']%86;steps.append(case);text=cases[case]
   if case==72:return dict(Type=event,Properties=props,Archive=True,Trace=steps,Labels=labels_seen)
  else:text=pending;pending=None
  for name,kind in re.findall(r'(\w+) = \((\w+)\)\(\(\w+ is \w+\) \? \w+ : null\);',text):live[name]=is_type(kind)
  while (m:=re.search(r'\bif \(',text)):
   # A goto before this condition leaves the current block immediately.
   if re.search(r'\bgoto ',text[:m.start()]):break
   start=m.end()-1;end=bracket(text,start);ys=text.index('{',end);ye=bracket(text,ys,'{','}')
   no=re.match(r'\s*else\s*{',text[ye:])
   if not no:raise RuntimeError('if without else at '+str(steps[-1]))
   ns=ye+no.end()-1;ne=bracket(text,ns,'{','}')
   selected=text[ys+1:ye-1] if cond(text[start+1:end-1]) else text[ns+1:ne-1]
   text=text[:m.start()]+selected+text[ne:]
  jump=re.search(r'\bgoto (\w+);|\breturn;',text)
  prefix=text[:jump.start()] if jump else text
  for name,value in re.findall(r'\b(num\d+) = ([^;]+);',prefix):v[name]=expr(value)
  if jump:
   label=jump[1]
   if label=='IL_0093':continue
   if label and label in labels:pending=labels[label];labels_seen.append(label);continue
   return dict(Type=event,Properties=props,Archive=False,Trace=steps,Labels=labels_seen,Exit=label or 'return')
  if 'num6' not in text:return dict(Type=event,Properties=props,Archive=False,Trace=steps,Labels=labels_seen,Exit='end')
 raise RuntimeError('dispatcher loop')
def all_property_paths(event,props=None):
 pending=[props or dict(Docked=False,OnFoot=False,SameSystem=False,SystemAddress=1)];outputs=[]
 while pending:
  properties=pending.pop()
  try:outputs.append(trace(event,properties))
  except OpaqueCondition as error:
   condition=str(error)
   for value in [False,True]:pending.append(properties|{'__predicates':properties.get('__predicates',{})|{condition:value}})
  if len(pending)+len(outputs)>128:raise RuntimeError('property path explosion')
 return outputs
if __name__=='__main__':
 if sys.argv[1:]==['--capture']:
  import hashlib
  def journal_type(kind):
   seen=set()
   while kind and kind not in seen:
    if kind=='JournalBase':return True
    seen.add(kind);kind=parents.get(kind,{}).get('Base')
   return False
  types=sorted({kind for kind in parents if journal_type(kind)}|{'CargoFile','NavRouteFile','MarketFile','OutfittingFile','ShipyardFile','BackpackFile','FCMaterialsFile','ModuleInfoFile','ShipLockerFile','FutureJournalEvent'})
  captures=[]
  for kind in types:captures.extend(all_property_paths(kind))
  for docked in [False,True]:
   for foot in [False,True]:
    for same in [False,True]:captures.append(trace('CarrierJump',dict(Docked=docked,OnFoot=foot,SameSystem=same,SystemAddress=1)))
  result=dict(SourceSHA256=hashlib.sha256(source.encode()).hexdigest(),SourceMethod='HecateWorker.JournalEvent<TJournal>.MoveNext (mod86)',AsyncModel='successful completed await; identical resumed continuation',PropertyModel='opaque non-type predicates explored both ways, no installed code execution',Captures=captures,ArchivedTypes=sorted({c['Type'] for c in captures if c['Archive']}))
  output=Path(__file__).resolve().parents[1]/'fixtures/hecate-archive-dispatch-oracle.json';output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(Cases=len(captures),ArchivedTypes=result['ArchivedTypes'])));raise SystemExit
 for kind in sys.argv[1:]:
  try:print(json.dumps(trace(kind,dict(Docked=True,SameSystem=False))))
  except Exception as error:print(json.dumps(dict(Type=kind,Error=str(error))))
