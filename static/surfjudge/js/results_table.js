(function($, undefined){
    $.widget('surfjudge.results_table', {
        options: {
            heat_id: null,
            getresultsurl: '/rest/results/{heatid}',
            getheaturl: '/rest/heats/{heatid}',

            websocket_url: null,
            websocket_channels: ['results'],

            show_header: true,
            show_wave_scores: true,
            show_needs: true,
            show_needs_first: true,
            show_needs_second: true,
            show_best_waves: false,
            fixed_column_width: false,
            big_numbers: false,
            abbreviate_first_name: false,
            abbreviate_wave_header: false,

            small: false,

            decimals: 2, // maximum (or exact) number of decimals
            fixed_decimals: true, // whether each number should have a fixed number of decimals e.g. 4.00
        },

        _create: function(){
            var _this = this;
            this.results = [];
            this.heat = {};

            if (this.options.websocket_url) {
                console.log('Initiating websocket for results table.')
                var channels = {};
                var on_message = function(msg){
                    var msg = JSON.parse(msg);
                    if (msg["heat_id"] == this.options.heat_id) {
                        console.log("refreshing");
                        this.refresh();
                    }
                };
                $.each(this.options.websocket_channels, function(idx, channel){
                    channels[channel] = on_message.bind(_this);
                });
                this.websocket = new WebSocketClient({
                    url: this.options.websocket_url,
                    channels: channels,
                    name: 'Results Table',
                });
            }

            this._init_html();
	        this._register_events();
            this.refresh();
        },

        _destroy: function(){
            if (this.websocket != null)
                this.websocket.close();
            this.element.empty();
            },

        _init_html: function(){
            var _this = this;
            var html = $([
                '<table class="table results_table borderless"></table>',
            ].join(' '));

            this.element.append(html);
        },

	    _register_events: function(){
	    },

        refresh: function(){
            var _this = this;
            var deferred_results = $.Deferred();
            $.get(this.options.getresultsurl.format({heatid: this.options.heat_id}))
                .done(function(data){
                    _this.results = data;
                    deferred_results.resolve();
                })
                .fail(function(){
                    console.log('Could not read results.');
                    deferred_results.resolve();
                });
            var deferred_heat = $.Deferred();
            $.get(this.options.getheaturl.format({heatid: this.options.heat_id}))
                .done(function(data){
                    _this.heat = data;
                    deferred_heat.resolve();
                })
                .fail(function(){
                    console.log('Could not read heat info.');
                    deferred_heat.resolve();
                });
            var deferred = $.Deferred();
            $.when(deferred_results, deferred_heat).done(function(){
                _this._refresh();
                deferred.resolve();
            });
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('table').empty();
            if (this.options.fixed_column_width) {
                this.element.find('table').addClass("fixed-width");
            }
            if (this.options.big_numbers) {
                this.element.find('table').addClass("big-numbers");
            }
            if (this.options.small) {
                this.element.find('table').addClass('table-sm');
            }

            // prepare score data
            var surfer_scores = new Map();
            var max_n_waves = 0;
            $.each(this.results, function(idx, s){
                surfer_scores.set(s['surfer_id'], s);
                max_n_waves = Math.max(max_n_waves, surfer_scores.get(s['surfer_id'])['wave_scores'].length);
            });

            var sorted_total_scores = $.map(this.results, function(surfer_result){
                return parseFloat(surfer_result['total_score']);
            }).concat().sort(function(a, b){return b - a});

            var best_waves = null;

            if (this.heat.heat_type == 'call') {
                best_waves = this._compute_best_waves_call();
            } else {
                best_waves = this._compute_best_waves();
            }

            // sort participants array
            var no_scores_available = false;
            if (this.heat['participations'].every(function(a) {
                var score_a = surfer_scores.get(a['surfer_id']) || {};
                return !('place' in score_a);
            })) {
                no_scores_available = true;
            };
            this.heat['participations'].sort(function(a,b){
                var score_a = surfer_scores.get(a['surfer_id']) || {};
                var score_b = surfer_scores.get(b['surfer_id']) || {};
                var val_a = score_a['place'];
                var val_b = score_b['place'];

                if (no_scores_available) {
                    val_a = a['seed'];
                    val_b = b['seed'];
                }
		        return val_a - val_b;
            });

            // write table header
            var header = $('<thead>');
            var row = $('<tr>')
                .append($('<td>', {html: '&nbsp;', class: 'color_header'}))
                .append($('<td>', {html: '', class: 'place_header'}))
                .append($('<td>', {html: 'Surfer', class: 'surfer_header'}))
                .append($('<td>', {html: 'Score', class: 'total_score_header'}));

            if (this.options.show_needs && this.heat.heat_type != 'call') {
                var needs_first = this._compute_needs(sorted_total_scores[0] || 0);
                var needs_second = this._compute_needs(sorted_total_scores[1] || 0);
                if (this.options.show_needs_first && this.options.show_needs_second) {
                    row.append($('<td>', {html: 'Needs 1./2.', class: 'needs_header'}));
                } else {
                    row.append($('<td>', {html: 'Needs', class: 'needs_header'}));
                }
            }

            if (this.options.show_best_waves && this.heat.heat_type != 'call') {
                row.append($('<td>', {html: 'Best Waves', class: 'best_waves_header'}));
            }

            if (this.options.show_wave_scores) {
                for (var i = 0; i < max_n_waves; i++){
                    row.append($('<td>', {text: this.options.abbreviate_wave_header ? 'W ' + (i+1) : 'Wave ' + (i+1), class: 'wave_score_header'}));
                };
            }
            if (this.options.show_header) {
                header.append(row);
            }

            // write table body
            var body = $('<tbody>');

            // after foregoing code, this.heat['participations'] is sorted by total score
            $.each(this.heat['participations'] || [], function(idx, participation){
                var sid = participation['surfer_id'];

                if (_this.options.show_needs && _this.heat.heat_type != 'call') {

                    // compile string for needs cell
                    var nf = needs_first.get(sid);
                    var ns = needs_second.get(sid);
                    var needs_data = {
                            nf: nf < 0 ? '-' : _this._float_str(nf),
                            ns: ns < 0 ? '-' : _this._float_str(ns),
                    };
                    if (_this.options.show_needs_first && _this.options.show_needs_second) {
                        var needs_str = "{nf} / {ns}".format(needs_data);
                    } else if (_this.options.show_needs_first) {
                        var needs_str = "{nf}".format(needs_data);
                    } else {
                        var needs_str = "{ns}".format(needs_data);
                    }
                }
                var result_data = surfer_scores.get(sid) || {'total_score': 0, 'wave_scores': []};
                var total_score_str = '--';
                if (max_n_waves > 0) {
                    if (_this.heat.heat_type == 'call') {
                        total_score_str = result_data['total_score'].toFixed(0);
                    } else {
                        total_score_str = _this._float_str(_this._round(result_data['total_score']));
                        best_waves_str = "{0} + {1}".format(
                            _this._float_str(_this._round(best_waves.get(sid)[0]["score"])),
                            _this._float_str(_this._round(best_waves.get(sid)[1]["score"]))
                        );
                    }
                }

                var row = $('<tr>', {
                    class: "surfer_{0}".format(sid),
                    style: "background-color:" + lighten(participation['lycra_color']['hex']), // the last two digits are the opacity
                })
                    .append($('<td>', {
                        html: '&nbsp;&nbsp;',
                        class: 'color_cell',
                        style: "background-color:" + participation['lycra_color']['hex'] + ";",
                    }))
                    .append($('<td>', {
                        text: max_n_waves == 0 ? '--' : (result_data['place'] == null ? 1 : (result_data['place'] + 1)) + '.',
                        class: 'place_cell',
                    }))
                    .append($('<td>', {
                        html: '<span class="first_name">{first_name}</span>{line_break} <span class="last_name">{last_name}</span>'.format({
                            first_name: _this.options.abbreviate_first_name ? participation['surfer']['first_name'].slice(0, 1) + "." : participation['surfer']['first_name'],
                            last_name: participation['surfer']['last_name'].length > 11 ? participation['surfer']['last_name'].toUpperCase().slice(0, 11) + "." : participation['surfer']['last_name'].toUpperCase(),
                            line_break: _this.options.small || _this.options.abbreviate_first_name ? '' : '<br>',
                        }),
                        class: 'name_cell',
                    }))
                    .append($('<td>', {
                        text: total_score_str,
                        class: result_data['unpublished'] ? 'total_score_cell unpublished' : 'total_score_cell',
                    }));

                if (_this.options.show_needs && _this.heat.heat_type != 'call') {
                    row.append($('<td>', {
                            text:  max_n_waves == 0 ? '--' : needs_str,
                            class: 'needs_cell',
                        }));
                }
                if (_this.options.show_best_waves && _this.heat.heat_type != 'call') {
                    row.append($('<td>', {
                            text:  max_n_waves == 0 ? '' : best_waves_str,
                            class: 'best_waves_cell',
                        }));
                }
                if (_this.options.show_wave_scores) {
                    for (var i = 0; i < max_n_waves; i++){
                        var score = result_data['wave_scores'].filter(function(s){
                            return (s['wave'] == i);
                        })[0] || null;
                        var val = '';
                        var classes = ['wave_score_cell', 'wave_{0}'.format(i)];
                        if (score !== null){
                            if (score['unpublished']){
                                classes.push('unpublished');
                            }
                                val = score['score'] < 0 ? 'M' : _this._float_str(_this._round(score['score']));
                        }
                        row.append($('<td>', {
                            text: val,
                            class: classes.join(' '),
                        }));
                    };
                }

                body.append(row);
            });

            //this.element.find('table').empty();
            this.element.find('.results_table').append(header).append(body);

            this._mark_best_waves(best_waves);
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

        _mark_best_waves: function(best_waves){
            var _this = this;
            best_waves.forEach(function(scores, surfer_id){
                $.each(scores, function(idx, score){
                    var selector = '.surfer_{0} .wave_{1}'.format(surfer_id, score['wave']);
                    _this.element.find(selector).addClass('best_wave');
                });
            });
        },

        _compute_best_waves: function(){
            var best_wave = new Map();
            $.each(this.results, function(idx, surfer){
                if ((surfer['wave_scores'] || []).length == 0) {
                    best_wave.set(surfer['surfer_id'], [{score: 0, wave: -1}, {score: 0, wave: -1}]);
                }
                // sort waves for surfer by score
                var sorted_ws = (surfer['wave_scores'] || []).concat().sort(function(a, b){
                    return b['score'] - a['score'];
                });
                // get best wave of surfer
                var bw = sorted_ws[0] || {score: 0, wave: -1};
                var sbw = sorted_ws[1] || {score: 0, wave: -1};
                best_wave.set(surfer['surfer_id'], [bw, sbw]);
            });
            return best_wave;
        },

        _compute_best_waves_call: function(){
            var _this = this;
            var best_waves = new Map();
            var scores_by_wave = new Map();
            $.each(this.results, function(idx, surfer){
                $.each(surfer['wave_scores'] || [], function(widx, score){
                    var wave = score['wave'];
                    if (!scores_by_wave.has(wave)) {
                        scores_by_wave.set(wave, []);
                    }
                    scores_by_wave.get(wave).push(score);
                });
            });
            scores_by_wave.forEach(function(scores, wave){
                var best = scores.sort(function(a, b){
                    return b['score'] - a['score'];
                })[0];
                var best_val = _this._round(best['score']);
                $.each(scores, function(idx, score){
                    if (_this._round(score['score']) == best_val) {
                        var surfer_id = score['surfer_id'];
                        if (!best_waves.has(surfer_id)) {
                            best_waves.set(surfer_id, []);
                        }
                        best_waves.get(surfer_id).push(score);
                    }
                });
            });
            return best_waves;
        },

        _compute_needs: function(target_total_score) {
            var _this = this;

            // round value to two decimals and add 0.01
            var exceed_round = function(val) {
                return _this._round(val) + 1.0/(10**_this.options.decimals);
            };
            // initialize needs with target_total_score
            // also for participants, that do not appear in this.results, yet
            var needs = new Map();
            $.each(this.heat['participations'], function(idx, part){
                var need = target_total_score > 0 ? _this._round(target_total_score) : -1;
                needs.set(part['surfer_id'], need);
            });

            $.each(this.results, function(idx, surfer){
                var wave_scores = surfer['wave_scores'].concat() || [];

                // sort waves for surfer by score
                var sorted_ws = wave_scores.sort(function(a, b){
                    return b['score'] - a['score'];
                });
                // get best wave of surfer
                var bw = sorted_ws[0] || {score: 0, wave: -1};

                if (surfer['total_score'] >= target_total_score - 0.001) {
                    needs.set(surfer['surfer_id'], -1);
                } else {
                    var need = _this._round(target_total_score - bw['score']);
                    needs.set(surfer['surfer_id'], need);
                }
            });
            return needs;
        },

        export_png: function(elem) {
            _this = this;
            var elem = elem || this.element[0];
            html2canvas(elem, {
                scrollX: -window.scrollX,
                scrollY: -window.scrollY,
            }).then(function(canvas){
                //transform div in canvas to output a png
                var png = canvas.toDataURL("image/png");

                // generate a download by simulating a click
                var link = document.createElement('a');
                document.body.appendChild(link);
                link.download = "{0}_{1}.png".format(
                    _this.heat["category"]["name"],
                    _this.heat["name"]
                );
                link.style = "display: none";
                link.href = png;
                link.click();
                link.remove();
            });
        },
    });
}(jQuery));
