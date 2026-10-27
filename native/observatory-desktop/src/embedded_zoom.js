(()=>{
  let old=document.getElementById('native-zoom-menu');if(old){old.remove();return;}
  const at=window.__observatoryPointer||[0,0],menu=document.createElement('div');menu.id='native-zoom-menu';
  menu.style=`box-sizing:border-box;position:fixed;left:${Math.min(at[0],Math.max(0,innerWidth-505))}px;top:${Math.min(at[1],Math.max(0,innerHeight-54))}px;width:505px;height:54px;z-index:2147483647;background:$$BACKGROUND$$;color:$$FOREGROUND$$;border:1px solid #696969;padding:6px;display:flex;gap:6px;align-items:center;font:13px sans-serif`;
  const slider=document.createElement('input');slider.type='range';slider.min=50;slider.max=600;slider.step=5;slider.value=parseFloat(document.body.style.zoom)||100;slider.style='width:365px';
  const number=document.createElement('span');number.style='width:45px';number.textContent=slider.value+'%';
  const set=n=>{slider.value=n;number.textContent=n+'%';location.href='observatory://bioinsights/nativezoom/'+n;};slider.oninput=()=>set(Number(slider.value));
  const reset=document.createElement('button');reset.textContent='Reset';reset.style='width:70px;height:37px';reset.onclick=()=>set(100);
  menu.append(slider,number,reset);document.documentElement.appendChild(menu);
})()
