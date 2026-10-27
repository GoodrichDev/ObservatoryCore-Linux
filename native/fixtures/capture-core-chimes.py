"""Read original Core resx bytes only; compare six native chime assets."""
import base64,hashlib,json,xml.etree.ElementTree as ET
from pathlib import Path
repo=Path(__file__).resolve().parents[2]
source=ET.parse('/home/loic/.cache/observatory-readable/core/Observatory.Assets.Resources.resx')
rows=[]
for item in source.findall('.//data'):
    name=item.attrib.get('name','')
    if not name.startswith('ObservatoryNotification'):continue
    data=base64.b64decode(item.findtext('value'));local=(repo/'ObservatoryCore/Assets'/(name+'.wav')).read_bytes()
    rows.append(dict(Name=name,SHA256=hashlib.sha256(data).hexdigest(),Bytes=len(data),MatchesNative=data==local))
rows.sort(key=lambda x:x['Name']);assert len(rows)==6 and all(x['MatchesNative']for x in rows)
(repo/'native/fixtures/core-chime-assets.json').write_text(json.dumps(rows,indent=2)+'\n')
