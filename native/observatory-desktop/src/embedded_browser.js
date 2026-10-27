// Host control events; content and navigation are supplied by the plugin.
window.__observatoryPopout=$$POPOUT$$;
document.querySelector('.buttonheader').style.display=$$HIDE_BUTTONS$$?'none':'';
document.addEventListener('pointerdown',e=>{
  window.__observatoryPointer=[e.clientX,e.clientY];
  for(const id of ['native-zoom-menu','observatory-popout-menu']){
    const menu=document.getElementById(id);
    if(menu&&!menu.contains(e.target))menu.remove();
  }
});
window.addEventListener('blur',()=>{for(const id of ['native-zoom-menu','observatory-popout-menu'])document.getElementById(id)?.remove();});
document.addEventListener('keydown',e=>{
  if(e.key==='Escape')for(const id of ['native-zoom-menu','observatory-popout-menu'])document.getElementById(id)?.remove();
});
// Disable the web engine's unrelated Back/Reload/Inspect menu. The source
// plugin provides the popout menu only when its control is detached.
document.addEventListener('contextmenu',e=>{
  e.preventDefault();document.getElementById('observatory-popout-menu')?.remove();
  if(!window.__observatoryPopout)return;
  const menu=document.createElement('div');menu.id='observatory-popout-menu';
  menu.style=`position:fixed;left:${e.clientX}px;top:${e.clientY}px;z-index:2147483647;background:#222;color:#d3d3d3;border:1px solid #696969;padding:3px;font:13px sans-serif;`;
  const add=(label,route)=>{const link=document.createElement('a');link.href='observatory://bioinsights/'+route;link.textContent=label;link.style='display:block;color:inherit;text-decoration:none;padding:4px 12px';link.onmouseover=()=>link.style.background='#696969';link.onmouseout=()=>link.style.background='';link.onclick=()=>menu.remove();menu.appendChild(link);};
  const navigation=document.createElement('details'),title=document.createElement('summary');title.textContent='Navigate';navigation.appendChild(title);menu.appendChild(navigation);
  for(const [label,route]of [['Main Window','main'],['Show Current Data','unsold'],['About','about'],['Codex Companion','codexcompanion']]){const first=menu.children.length;add(label,route);navigation.appendChild(menu.children[first]);}
  for(const [label,command]of [['Toggle OnTop','ToggleOnTop'],['Lock/Unlock Position','ToggleTitleBar'],['Toggle Buttons','ToggleButtons'],['Force Save Position','SavePopoutPosition'],['Close Popout','Popout']])add(label,'nativeaction/'+command);
  document.documentElement.appendChild(menu);
});
