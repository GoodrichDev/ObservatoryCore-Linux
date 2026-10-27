// Connect to WebSocket
var ws;
const usews = $$USE_WS$$;
var showdiscon = true;
if (usews) {
    ws = new WebSocket(`ws://$$WS_SERVER$$/$$PLUGIN$$`);

    ws.onopen = () => console.log("WS connected");

    ws.onmessage = function (event) {
        const response = JSON.parse(event.data);
        //console.log('Server response:', response);
        if (response.event == "reload")
            window.location.reload();
        if (response.event == "reloadpage") {
            if (response.page == window.location.pathname)
                window.location.reload();
        }
        else if (response.event == "hide")
            document.getElementById(response.id).style.display = 'none';
        else if (response.event == "readall") {
            document.getElementById('webapp').innerHTML = '<div class="alert alert-info">Read All in progress</div><br />';
            document.getElementById('webapp').style.display = 'inline';
        }
        else if (response.event == "content") {
            var div = document.getElementById(response.id);
            if (div != null) {
                document.getElementById(response.id).innerHTML = response.content;
                onLoad();
            }
        }
    };
    ws.onclose = function () {
        console.log('Disconnected from server');
        //do on small delay to stop refresh flashing disconnected
        if (showdiscon) {
            let timer = setInterval(disconnect, 500);
        }
    };

    ws.onerror = function (error) {
        console.error('WebSocket error:', error);
    };


    document.addEventListener('click', e => {
        {
            if (!usews) return;
            const a = e.target.closest('a');
            if (!a) return;
            showdiscon = false;
            if (a.href.startsWith('http://$$WS_SERVER$$')) {
                {
                    e.preventDefault();
                    console.log('click:', a.href);
                    ws.send(a.href);
                }
            }
        }
    });
}

function disconnect() {
    document.getElementById('disconnected').innerHTML = '<div class="alert alert-danger">Disconnected</div><br />';
    document.getElementById('disconnected').style.display = 'inline';
}

function onLoad() {
    document.getElementById('popout').style.display = 'none';
    document.getElementById('zoom').style.display = 'none';
    document.getElementById('settings').style.display = 'none';
}

window.onload = onLoad;
