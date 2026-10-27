#!/usr/bin/env python3
"""Regenerate the source CSS captures for both original themes using the existing case list."""
import argparse,json,os,subprocess,tempfile
from pathlib import Path

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("worker",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--windows-fonts",action="store_true")
    args=parser.parse_args()
    fixture=Path(__file__).parents[1]/"fixtures/bioinsights-species-css.json"
    source=json.loads(fixture.read_text(encoding="utf-8"))
    args.output.mkdir(parents=True,exist_ok=True)
    source=[{"page":page,"width":width,"theme_index":theme} for page,width in dict.fromkeys((case["page"],case["width"])for case in source) for theme in [0,1]]
    source.extend({"page":"codexcompanion/16/91","width":1168,"theme_index":theme,"gallery_caption":True} for theme in [0,1])
    cases=[]
    for case in source:
        directory=args.output/(case["page"].replace("/","-")+"-"+str(case["width"])+"-theme"+str(case["theme_index"])+("-caption" if case.get("gallery_caption") else ""))
        command=["python3",str(Path(__file__).with_name("bioinsights_species_probe.py")),str(args.worker),"--page",case["page"],"--theme-index",str(case["theme_index"]),"--width",str(case["width"]),"--output",str(directory)]
        if case.get("gallery_caption"):command.append("--gallery-caption")
        if args.windows_fonts:command.append("--windows-fonts")
        subprocess.run(command,check=True)
        cases.append(json.loads((directory/"dom.json").read_text(encoding="utf-8")))
    destination=args.output/"bioinsights-species-css.json"
    destination.write_text(json.dumps(cases,indent=2)+"\n",encoding="utf-8")
    # Windows fonts are an explicitly separate source-oracle mode; do not change
    # native-system fixture expectations merely to hide a different font stack.
    if not args.windows_fonts:
        temporary=fixture.with_suffix(".json.tmp")
        temporary.write_text(destination.read_text(encoding="utf-8"),encoding="utf-8")
        os.replace(temporary,fixture)
    print(json.dumps({"cases":len(cases),"mode":"system+Windows" if args.windows_fonts else "system","fixture":str(destination)}))
if __name__=="__main__":main()
