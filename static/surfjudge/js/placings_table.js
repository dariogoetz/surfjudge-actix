/* =========================================================
 * placings_table.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.placings_table', {
        options: {
            heat_id: null,
            getheaturl: '/rest/heats/{heatid}',
            getresultsurl: '/rest/preliminary_results/{heatid}',
            getadvancementsurl: '/rest/advancements?from_heat_id={fromheatid}',
            getparticipantsurl: '/rest/participants/{heatid}/{seed}',
            postparticipantsurl: '/rest/participants/{toheatid}/{seed}',

            websocket_url: null,
            websocket_channels: ['scores', 'results', 'participants'],

            show_header: false,

            decimals: 2,
            fixed_decimals: true, // whether each number should have a fixed number of decimals e.g. 4.00
        },

        _create: function(){
            var _this = this;
            this.data_heat = null;
            this.data_results = null;
            this.results_map = null;
            this.data_advancements = null;
            this.advancements_map = null;

            if (!this._check_inputs()){
                console.log('Inputs of placings_table module not valid.');
                return;
            }

            console.log('Initiating websocket for placings table.')
            var on_msg = function(msg) {
                var msg = JSON.parse(msg);
                var heat_id = msg["heat_id"];
                if (_this.options.heat_id == heat_id) {
                    console.log("Placings table: Something changed for this heat. Refreshing.");
                    _this.refresh();
                } else {
                    console.log("Placings table: Something changed for some other heat. Not refreshing.");
                }
            };
            var channels = {};
            $.each(this.options.websocket_channels, function(idx, channel){
                channels[channel] = on_msg;
            });
            this.websocket = new WebSocketClient({
                url: this.options.websocket_url,
                channels: channels,
                name: 'Placings Table',
            });

            this._init_html();

            this._register_events();
            if (this.options.heat_id !== null)
                this._initialized = this.refresh();
            else
                this._initialized = $.Deferred().resolve().promise();
        },

        _destroy: function(){
            if (this.websocket != null)
                this.websocket.close();
            this.element.empty();
            },

        _check_inputs: function(){
            return true;
        },

        _init_html: function(){
            var _this = this;
            var html = [
                '<table class="table placings_table"></table>',
            ].join(' ');

            this.element.append(html);
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .advance_btn': this._advance_surfer,
            });
        },

        refresh: function(){
            console.log('Placings table: Refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.heat_id !== null){
                var def_advancements = $.Deferred();
                $.getJSON(this.options.getadvancementsurl.format({fromheatid: this.options.heat_id}))
                    .done(function(advancements){
                        _this.data_advancements = advancements;
                        var adv_map = new Map();
                        var deferreds = [];
                        $.each(_this.data_advancements, function(idx, adv){
                            adv_map.set(adv['place'], adv);
                            // see if advancement target already has a participant
                            var def = $.getJSON(_this.options.getparticipantsurl.format({heatid: adv['to_heat_id'], seed: adv['seed']}), function(part){
                                adv['seed_is_free'] = (part.length == 0);
                            });
                            deferreds.push(def);
                        });
                        _this.advancement_map = adv_map;
                        $.when.apply($, deferreds).then(function(){
                            def_advancements.resolve();
                        });
                    })
                    .fail(function(){
                        console.error('Connection error to get advancements.');
                        def_advancements.resolve();
                    });

                var def_heat = $.Deferred();
                $.get(this.options.getheaturl.format({heatid: this.options.heat_id}))
                    .done(function(data){
                        _this.data_heat = data;
                        def_heat.resolve();
                    })
                    .fail(function(){
                        console.log('Could not read heat info.');
                        def_heat.resolve();
                    });

                var def_results = $.Deferred();
                $.getJSON(this.options.getresultsurl.format({heatid: this.options.heat_id}))
                    .done(function(results){
                        if (results != null){
                            _this.data_results = results;
                            _this.results_map = new Map();
                            $.each(results, function(idx, result){
                                _this.results_map.set(result['surfer_id'], result);
                            });
                            def_results.resolve();
                        } else {
                            console.error('Heat not found.');
                            _this.options.heat_id = null;
                            def_results.resolve();
                        }
                    })
                    .fail(function(){
                        console.error('Connection error.');
                        def_results.reject();
                    });
                $.when(def_advancements, def_results, def_heat).done(function(){
                    _this._refresh();
                    deferred.resolve();
                });
            } else {
                console.log('Nothing to refresh (no heat id specified)');
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('table').empty();

            // TABLE HEADER
            if (this.options.show_header) {
                var header = $('<thead>');
                var row = $('<tr>')
                    .append($('<td>', {html: 'Place', class: 'place_cell'}))
                    .append($('<td>', {html: 'Surfer', class: 'name_cell'}))
                    .append($('<td>', {html: 'Score', class: 'total_score_cell'}))
                    .append($('<td>', {html: 'Advance', class: 'advance_cell'}));
                header.append(row);
                this.element.find('.placings_table').append(header);
            }

            // TABLE BODY
            var body = $('<tbody>');
            this.data_heat['participations'].sort(function(a, b){
                var res_a = _this.results_map.get(a['surfer_id']) || {place: a['seed']};
                var res_b = _this.results_map.get(b['surfer_id']) || {place: b['seed']};
                return res_a['place'] - res_b['place'];
            });

            $.each(this.data_heat['participations'], function(idx, participation){
                var result_data = _this.results_map.get(participation['surfer_id']);
                var surfer_id = result_data['surfer_id'];
                var place = result_data['place'];
                var lycra_color_id = participation['lycra_color_id'];
                var adv_data = _this.advancement_map.get(place) || {};
                var adv_html = '';
                if (adv_data['seed_is_free'] && !result_data['unpublished'] && result_data['wave_scores'].length > 0) {
                    adv_html = [
                        '<button class="btn btn-success advance_btn">',
                        '<span class="fa fa-angle-double-up"></span>',
                        '&nbsp;{0}<br>Seed {1}'.format(adv_data['to_heat']['name'], adv_data['seed'] + 1),
                        '</button>'].join('');
                }
                var row = $('<tr>', {
                    class: "place_{0}".format(result_data['place']),
                    style: "background-color:" + lighten(participation['lycra_color']['hex']), // the last two digits are the opacity
                })
                    .append($('<td>', {
                        text: (place + 1) + '.',
                        class: 'place_cell',
                        style: ""
                    }))
                    .append($('<td>', {
                        html: result_data['surfer']['first_name'] + ' ' + result_data['surfer']['last_name'],
                        class: 'name_cell',
                    }))
                    .append($('<td>', {
                        text: _this._float_str(_this._round(result_data['total_score'])),
                        class: result_data['unpublished'] ? 'total_score_cell unpublished' : 'total_score_cell',
                    }))
                    .append($('<td>', {
                        html: adv_html,
                        class: 'advancement_cell',
                        data: {surfer_id: surfer_id, place: place, lycra_color_id: lycra_color_id},
                    }));
                body.append(row);
            });


            this.element.find('.placings_table').append(body);
        },

        _round: function(val, precision) {
            if (precision == null) {
                var precision = this.options.decimals;
            }
            return (Math.round(val * 10**precision) / 10**precision);
        },

        _float_str: function(val){
            if (this.options.fixed_decimals) {
                return val.toFixed(this.options.decimals);
            } else {
                return +parseFloat(val);
            }
        },

        _advance_surfer: function(ev) {
            var _this = this;
            var td_elem = $(ev.target).closest('td');
            var place = td_elem.data('place');
            var surfer_id = td_elem.data('surfer_id');
            var adv_data = this.advancement_map.get(place);
            var to_heat_id = adv_data['to_heat_id'];
            var target_seed = adv_data['seed'];
            var data = {surfer_id: surfer_id};
            $.getJSON(this.options.getheaturl.format({heatid: to_heat_id}), function(heat){
                if (heat.heat_type == 'call') {
                    // use same color as in current heat if the next heat is a call
                    data['lycra_color_id'] = td_elem.data('lycra_color_id');
                }
                $.post(_this.options.postparticipantsurl.format({seed: target_seed, toheatid: to_heat_id}),
                    JSON.stringify(data), function(){
                        _this.refresh();
                });
                $(ev.target).remove();
            });
        },

        upload: function(){
            // no upload function?
        },


        _check_data: function(){
            // no user input, therefore no check_data?
        },
    });
}(jQuery));
