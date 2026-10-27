#!/usr/bin/env python3
"""Decode an owner's supplied plugin constant table without executing its IL.

Inputs are an offline IL listing, the field-RVA bytes, the initializer's seed,
and an ILSpy output directory. Only the explicitly selected output is written.
"""
import argparse
import json
import lzma
from pathlib import Path
import re
import struct


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--il",type=Path,required=True)
    parser.add_argument("--rva",type=Path,required=True)
    parser.add_argument("--seed",type=int,required=True)
    parser.add_argument("--source",type=Path,required=True)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    raw=args.rva.read_bytes()
    words=struct.unpack("<"+"I"*(len(raw)//4),raw)
    seed=args.seed&0xffffffff
    keys=[]
    for _ in range(16):
        seed^=seed>>12;seed^=(seed<<25)&0xffffffff;seed^=seed>>27;seed&=0xffffffff;keys.append(seed)
    clear=bytearray()
    for start in range(0,len(words),16):
        for index in range(16):
            value=words[start+index]^keys[index]
            clear.extend(struct.pack("<I",value));keys[index]^=value
    props=clear[0]
    dictionary,length=struct.unpack_from("<II",clear,1)
    assert length<=64*1024*1024 and dictionary<=1024*1024*1024
    decoder=lzma.LZMADecompressor(format=lzma.FORMAT_RAW,filters=[dict(id=lzma.FILTER_LZMA1,dict_size=dictionary,lc=props%9,lp=(props//9)%5,pb=props//45)])
    table=decoder.decompress(bytes(clear[9:]),max_length=length)
    assert len(table)==length
    listing=args.il.read_text(encoding="utf-8-sig")
    sections=re.split(r"(?=^METHOD )",listing,flags=re.M)
    decoders={}
    for section in sections:
        header=section.splitlines()[0] if section else ""
        name=re.search(r"^METHOD 0x0600000[4-8] .<Module>::(\S+) ",header)
        if not name:continue
        arithmetic=re.search(r"ldarg\.0\s*\n IL_\w+: ldc\.i4 (-?\d+)\n IL_\w+: mul\s*\n IL_\w+: ldc\.i4 (-?\d+)\n IL_\w+: xor",section)
        assert arithmetic,header
        readable="".join("_"+format(ord(character),"04X") for character in name[1])
        decoders[readable]=(int(arithmetic[1]),int(arithmetic[2]))
    count=0;values={}
    expression=re.compile(r"_003CModule_003E\.([_0-9A-F]+)<string>\((-?\d+)\)")
    def substitute(match):
        nonlocal count
        if match[1] not in decoders:return match[0]
        multiplier,xor=decoders[match[1]]
        encoded=((int(match[2])*multiplier)&0xffffffff)^(xor&0xffffffff)
        offset=(encoded&0x3fffffff)<<2
        if offset+4>len(table):return match[0]
        size=struct.unpack_from("<I",table,offset)[0]
        if size>1024*1024 or offset+4+size>len(table):return match[0]
        try:value=table[offset+4:offset+4+size].decode("utf-8")
        except UnicodeDecodeError:return match[0]
        count+=1;values[match[0]]=value
        return json.dumps(value,ensure_ascii=False)
    args.output.mkdir(parents=True,exist_ok=True)
    for source in args.source.rglob("*.cs"):
        target=args.output/source.relative_to(args.source)
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(expression.sub(substitute,source.read_text(encoding="utf-8-sig")),encoding="utf-8")
    (args.output/"decoded-constants.json").write_text(json.dumps(values,ensure_ascii=False,indent=2),encoding="utf-8")
    print(json.dumps({"decoded_bytes":len(table),"decoder_count":len(decoders),"substitutions":count,"distinct_values":len(values)}))


if __name__=="__main__":main()
