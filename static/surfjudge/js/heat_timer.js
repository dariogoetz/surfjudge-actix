(function($, undefined){
    $.widget('surfjudge.heat_timer', {
        options: {
            heat_id: null,
            heat_duration: null,
            heat_state: null,

            // will be a Date object in case the heat state is "active", else null
            heat_end_time: null,

            getheatsurl: '/rest/heats/{heatid}',
            getheatstateurl: '/rest/heat_state/{heatid}',
            getremainingheattimeurl: '/rest/remaining_heat_time/{heatid}',

            websocket_url: null,

            class: "",
        },

        _create: function(){
            var _this = this;
            // return value from server for remaining time
            this._remaining_heat_time_at_load_s = null;

            // set interval timer
            this._heat_timer = null;

            // initialize widget
            this._init_html();
            this.refresh();

            console.log('Initiating websocket for heat timer.')
            var on_msg = function(msg){
                var msg = JSON.parse(msg);
                var heat_id = msg["heat_id"];
                var activity = msg["msg"];
                if (heat_id == this.options.heat_id) {
                    console.log("Heat timer: Refreshing");
                    _this.refresh();

                } else {
                    console.log("Heat timer: Some other heat's activity changed: not refreshing");
                }
            }
            this.websocket = new WebSocketClient({
                url: this.options.websocket_url,
                channels: {
                    'active_heats': this.refresh.bind(this),
                },
                name: 'Heat Timer',
            });
        },

        _destroy: function(){
            clearInterval(this._heat_timer);
            if (this.websocket != null)
                this.websocket.close();
            this.element.empty();
            },

        _init_html: function(){
            var _this = this;
            html = $([
                '<span class="heat_time">00:00</span>',
            ].join(' '));

            html.addClass(this.options.class);

            this.element.append(html);
        },

        refresh: function(){
            var _this = this;
            var deferred = $.Deferred();

            var deferred_heat_info = $.getJSON(this.options.getheatsurl.format({heatid: this.options.heat_id}))
                .done(function(ev_heat_info) {
                    _this.options.heat_duration = ev_heat_info['duration'];
                });
            var deferred_heat_state = $.getJSON(this.options.getheatstateurl.format({heatid: this.options.heat_id}))
                .done(function(heat_state) {
                    _this.options.heat_state = heat_state['state'];
                })
            var deferred_remaining = $.getJSON(this.options.getremainingheattimeurl.format({heatid: this.options.heat_id}))
                .done(function(remaining_heat_time_s){
                    _this._remaining_heat_time_at_load_s = remaining_heat_time_s;
                })
                .fail(function(){
                    _this.options.heat_end_time = null;
                });
                $.when(deferred_heat_info, deferred_heat_state, deferred_remaining)
                    .done(function(){
                        _this._refresh();
                        deferred.resolve();
                    })
                    .fail(function(){
                        _this._refresh();
                        deferred.reject();
                    });
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            if (this.options.heat_state === 'active') {
                var now = new Date();
                if (this._remaining_heat_time_at_load_s !== null){
                    _this.options.heat_end_time = new Date(now.getTime() + _this._remaining_heat_time_at_load_s * 1000);
                }
                if (this._heat_timer !== null)
                    clearInterval(this._heat_timer);
                this._heat_timer = setInterval(function(){_this._update_heat_time_display();}, 1000);
            }
            this._update_heat_time_display();
        },

        _update_heat_time_display: function(){
            var heat_time = Math.ceil(this._remaining_heat_time_at_load_s);
            if (this.options.heat_state === 'inactive'){
                // heat is inactive
                var time_str = (this.options.heat_duration === null) ? '--:--' : (this.options.heat_duration + ':00');
                this.element.find('.heat_time').text(time_str);
            } else if (this.options.heat_state === 'active') {
                // heat is active --> compute heat_time from heat_end_time
                var now = new Date();
                var heat_time = Math.ceil((this.options.heat_end_time.getTime() - now.getTime())/1000);
                if (heat_time < 0)
                    heat_time = 0;
            }
            var minutes = '' + parseInt(heat_time / 60);
            var seconds = '' + parseInt(heat_time % 60);
            while (seconds.length < 2) seconds = '0' + seconds;
            while (minutes.length < 2) minutes = '0' + minutes;
            var timer_elem = this.element.find('.heat_time');
            timer_elem.text(minutes + ':' + seconds);

            if (this.options.heat_state === 'paused') {
                timer_elem.addClass('paused');
            } else {
                timer_elem.removeClass('paused');
            }
        },
    });

}(jQuery));
