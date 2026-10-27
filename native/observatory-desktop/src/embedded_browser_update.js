(() => {
    const parsed = new DOMParser().parseFromString($$DOCUMENT$$, 'text/html');
    document.head.innerHTML = parsed.head.innerHTML;
    document.body.style.cssText = parsed.body.style.cssText;
    const content = document.getElementById('maincontent');
    // The source ccmark/ccshow actions replace individual elements and leave
    // the chosen gallery image and credit intact. This is a same-route update;
    // a different species/page loads a new document through the native host.
    const gallery = ['galleryimage', 'gallerycredit'].map(id => document.getElementById(id));
    content.innerHTML = parsed.getElementById('maincontent').innerHTML;
    for (const previous of gallery) {
        const replacement = previous && document.getElementById(previous.id);
        if (replacement) replacement.replaceWith(previous);
    }
    for (const previous of content.querySelectorAll('script')) {
        const script = document.createElement('script');
        script.textContent = previous.textContent;
        previous.replaceWith(script);
    }
})()
