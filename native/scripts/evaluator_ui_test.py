#!/usr/bin/env python3
"""Actual native Evaluator inline-page and custom-popout input in private Xvfb."""
import argparse,hashlib,json,subprocess,tempfile,time
from pathlib import Path
from native_update_test import Profile,wait
from x11_qa import PrivateDisplay

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    binary=args.binary.resolve();display=PrivateDisplay()
    try:
        with tempfile.TemporaryDirectory(prefix="observatory-evaluator-ui-") as temporary:
            profile=Profile(Path(temporary),"profile")
            (profile.plugins/"update-qa.lua").unlink()
            profile.settings.update(auto_start=True,start_read_all=True,popup=False,speech=False)
            profile.settings["plugins"]["StellarOverlay"]["DisplayOverride"]=False
            profile.save();env=profile.env|display.env
            for variable in ("XDG_CONFIG_HOME","XDG_DATA_HOME","XDG_CACHE_HOME"):
                env[variable]=profile.env[variable]
            env["HOME"]=str(profile.root)
            log=(args.output/"stderr.log").open("w")
            process=subprocess.Popen([str(binary),"--config",str(profile.config),"--view","Evaluator","--window-size","1200x850"],env=env,stdout=log,stderr=log)
            try:
                def main_window():
                    return next((window for window,details in display.x.windows().items() if details["mapped"] and details["width"]==1200),None)
                window=wait(main_window,"native Evaluator window")
                time.sleep(2)
                image=display.x.capture(window);image.save(args.output/"main.png")
                # Original six source buttons have 32px faces and 38px pitch.
                # Locate their upper-left corner from actual source theme pixels.
                face=(255,111,0)
                hits=[]
                for y in range(20,100):
                    for x in range(5,65):
                        if image.getpixel((x,y))==face and image.getpixel((x+28,y))==face and image.getpixel((x,y+28))==face:
                            hits.append((x,y))
                assert hits,"Evaluator source toolbar missing"
                x,y=hits[0];cx,cy=x+14,y+14
                receipt={"binary_sha256":hashlib.sha256(binary.read_bytes()).hexdigest(),"synthetic_profile":True,"display":"private Xvfb","toolbar_first_center":[cx,cy],"interactions":[]}
                def click(index,name):
                    display.x.click(window,cx+38*index,cy);time.sleep(.45)
                    visible={key:value for key,value in display.x.windows().items() if value["mapped"]}
                    display.x.capture(window).save(args.output/(name+".png"))
                    return visible
                baseline={key for key,value in display.x.windows().items() if value["mapped"]}
                about=click(2,"about");assert set(about)==baseline,about
                receipt["interactions"].append("About renders inline without creating a window")
                unsold=click(1,"unsold");assert set(unsold)==baseline,unsold
                receipt["interactions"].append("Show Current Data renders inline without creating a window")
                main=click(0,"main-return");assert set(main)==baseline,main
                receipt["interactions"].append("Main restores the source main page inline")
                zoom=click(4,"zoom");assert set(zoom)==baseline,zoom
                receipt["interactions"].append("Zoom opens the source in-window control")
                # Dismiss the zoom control before testing intentional source Popout.
                display.x.click(window,800,cy+80);time.sleep(.2)
                popout=click(3,"popout")
                popout=wait(lambda:{key:value for key,value in display.x.windows().items() if value["mapped"] and key not in baseline},"source custom Popout")
                assert len(popout)==1,popout
                receipt["interactions"].append("Only the explicit Popout button creates one custom window")
                target=next(iter(popout));display.x.capture(target).save(args.output/"custom-popout.png")
                (args.output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
                print(json.dumps(receipt))
            finally:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()
                log.close()
    finally:display.close()
if __name__=="__main__":main()
