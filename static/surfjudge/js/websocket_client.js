var WebSocketClient = function(options){
    var defaults = {
        url: 'ws://localhost:6544',
        channels: {},
        name: '',
        url_postfix: '/websocket', // if no url is given, use the host plus this postfix
    };
    this.options = $.extend(defaults, options);

    this.websocket = null;

    this.init();
}

WebSocketClient.prototype = {
    constructor: WebSocketClient,

    init: function(){
        var _this = this;
        var deferred = $.Deferred();

        if (this.options.url == null || this.options.url == 'null' || this.options.url == '') {
            var loc = window.location, new_uri;
            if (loc.protocol === "https:") {
                new_uri = "wss:";
            } else {
                new_uri = "ws:";
            }
            new_uri += "//" + loc.host;
            new_uri += this.options.url_postfix;
            this.options.url = new_uri;
        }

        // generate websocket
        this.websocket = new ReconnectingWebSocket(this.options.url);

        // register dispatcher with websocket
        this.websocket.onmessage = function(event){
            _this.dispatch(JSON.parse(event.data));
        };

        this.websocket.onopen = function(){
            // subscribe to all channels specified in options
            $.each(_this.options.channels, function(key){
                _this.subscribe(key);
            });
            deferred.resolve();
        }

        this.websocket.onclose = function(event) {
            if (event.code == 3001 || event.type == "close") {
                console.log('Websocket connection was closed: '+ _this.options.name);
            } else {
                console.log('Websocket connection error: ' + _this.options.name);
            }
        }
        this.websocket.onerror = function(error){
            console.log('Error in websocket connection: ' + _this.options.name);
        }
        this.initialized = deferred.promise();
    },

    subscribe: function(channel){
        console.log('Subscribing to websocket channel ' + channel + ': ' + this.options.name);
        this.websocket.send(JSON.stringify({'action': 'subscribe', 'channel': channel}));
    },

    dispatch: function(data){
        var channel = data['channel'];
        var consumer = this.options.channels[channel];
        if (consumer){
            this.initialized.done(function(){
                consumer(data['message']);
            });
        }
    },

    close: function(code, reason) {
        this.websocket.close(code, reason);
    },
}
