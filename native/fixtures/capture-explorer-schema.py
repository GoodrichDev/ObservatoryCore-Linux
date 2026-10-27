"""Capture public data properties from decompiled installed Framework DTOs."""
from pathlib import Path
import json,re,hashlib
root=Path('/home/loic/.cache/observatory-readable/framework')
types={};hashes={}
for folder in ['Observatory.Framework.Files.Journal','Observatory.Framework.Files.ParameterTypes']:
 for path in (root/folder).glob('*.cs'):
  text=path.read_text();declaration=re.search(r'public (?:sealed )?class (\w+)(?:\s*:\s*(\w+))?',text)
  if not declaration:continue
  properties={}
  for m in re.finditer(r'^\s*public\s+([^\n{]+?)\s+(\w+)\s*(?=\{|=>|\n\s*\{)',text,re.M):
   type_name,name=m.groups()
   if ' class ' in ' '+type_name+' ' or type_name.startswith(('static ','override ','class ')):continue
   properties[name]=type_name.strip()
  types[declaration[1]]=dict(Base=declaration[2],Properties=properties)
  hashes[declaration[1]]=hashlib.sha256(text.encode()).hexdigest()
out=Path(__file__).resolve().with_name('explorer-dto-schema.json');out.write_text(json.dumps(dict(Types=types,SourceHashes=hashes),ensure_ascii=False,separators=(',',':')),encoding='utf-8')
print(len(types))
