#!/usr/bin/env python3
"""Run the actual X11 owner lookup against nested native decoration frames."""
import argparse,hashlib,json,subprocess
from pathlib import Path
from x11_qa import PrivateDisplay

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('test_binary',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    private=PrivateDisplay()
    try:
        result=subprocess.run([str(args.test_binary.resolve()),'--ignored','--exact','embedded_browser::tests::locates_real_client_below_reparenting_frames_with_and_without_ewmh'],env=private.env,capture_output=True,text=True,timeout=20)
        assert result.returncode==0,result.stdout+result.stderr
        assert '1 passed' in result.stdout,result.stdout
        receipt={'binary_sha256':hashlib.sha256(args.test_binary.read_bytes()).hexdigest(),'checks':['nested actual X11 windows without EWMH','actual _NET_CLIENT_LIST client lookup','cached matching owner','destroyed owner is rejected'],'stdout':result.stdout}
        (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
        print(json.dumps(receipt))
    finally:private.close()

if __name__=='__main__':main()
