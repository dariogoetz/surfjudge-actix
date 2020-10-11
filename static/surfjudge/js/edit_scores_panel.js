(function($, undefined){
    $.widget('surfjudge.edit_scores_panel', {
        options: {
            heat_id: null,

            getheatsurl: '/rest/heats/{heatid}',
            getassignedjudgesurl: '/rest/judge_assignments/{heatid}',
            getparticipantsurl: '/rest/participants/{heatid}',
            getscoresurl: '/rest/scores',
            getpreliminaryresultsurl: '/rest/preliminary_results/{heatid}',

            websocket_url: null,

            mark_conspicious: true,
            mark_extremal: true,
        },

        _create: function(){
            var _this = this;

            this.heat = null;
            this.scores = null;
            this.prelim_results = null;
            this.judge_assignment = null;
            this.participant_info = null;

            console.log('Initiating websocket for edit scores panel.')
            var on_msg = function(msg) {
                var msg = JSON.parse(msg);
                var heat_id = msg["heat_id"];
                if (_this.options.heat_id == heat_id) {
                    console.log("Edit scores: Scores for this heat changed. Refreshing.");
                    _this.refresh();
                } else {
                    console.log("Edit scores: Some other heat's scores changed. Not refreshing.");
                }
            };
            this.websocket = new WebSocketClient({
                url: this.options.websocket_url,
                channels: {
                    'scores': on_msg,
                },
                name: 'Edit Scores Panel',
            });

            this._init_html();
            this._register_events();

            if (this.options.data != null){
                this.initialized = this.load(this.options.data);
            } else {
                this.initialized = this.refresh();
            }
            this.initialized.done(function(){
            });
        },

        _destroy: function(){
            if (this.websocket != null)
                this.websocket.close();
            this.element.empty();
            },

        _init_html: function(){
            var _this = this;
            html = $([
                '<div class="edit_scores_table">',
                '  <table class="table table-bordered table-sm">',
                '  </table>',
                '</div>',
            ].join(' '));

            this.element.append(html);
        },

        load: function(data_heat, data_judge, data_participants, data_scores, data_prelim_results){
            var _this = this;
            var deferred = $.Deferred();
            this.heat = data_heat;

            this.judge_assignments = data_judge;

            this.participant_info = new Map();
            $.each(data_participants, function(idx, p){
                _this.participant_info.set(parseInt(p['surfer_id']), p);
            });

            this.prelim_results = new Map();
            $.each(data_prelim_results, function(idx, r){
                // sort wave_scores by wave number
                r['wave_scores'].sort(function(a, b){return a['wave'] - b['wave']});
                _this.prelim_results.set(parseInt(r['surfer_id']), r);
            });

            this.scores = data_scores;
            this._annotate_scores();

            this._refresh();
            deferred.resolve();
            return deferred.promise();
        },

        refresh: function(){
            var _this = this;
            // load data from server and refresh
            var deferred = $.Deferred();

            var def_heat = $.Deferred();
            $.getJSON(this.options.getheatsurl.format({heatid: this.options.heat_id}))
                .done(function(data){def_heat.resolve(data);})
                .fail(function(){
                    console.error('Failed to load heat data');
                    def_heat.resolve();
                });
            var def_judge = $.Deferred();
            $.getJSON(this.options.getassignedjudgesurl.format({heatid: this.options.heat_id}))
                .done(function(data){def_judge.resolve(data);})
                .fail(function(){
                    console.error('Failed to load judge data');
                    def_judge.resolve();
                });
            var def_part = $.Deferred();
            $.getJSON(this.options.getparticipantsurl.format({heatid: this.options.heat_id}))
                .done(function(data){def_part.resolve(data);})
                .fail(function(){
                    console.error('Failed to load participants data');
                    def_part.resolve();
                });
            var def_scores = $.Deferred();
            $.getJSON(this.options.getscoresurl, {heat_id: this.options.heat_id})
                .done(function(data){def_scores.resolve(data);})
                .fail(function(){
                    console.error('Failed to load scores data');
                    def_scores.resolve();
                });
            var def_prelim_results = $.Deferred();
            $.getJSON(this.options.getpreliminaryresultsurl.format({heatid: this.options.heat_id}))
                .done(function(data){def_prelim_results.resolve(data);})
                .fail(function(){
                    console.error('Failed to load scores data');
                    def_prelim_results.resolve();
                });

            $.when(def_heat, def_judge, def_part, def_scores, def_prelim_results)
                .done(function(data_heat, data_judge, data_part, data_scores, data_prelim_results){
                    _this.load(data_heat, data_judge, data_part, data_scores, data_prelim_results).done(function(){
                        deferred.resolve();
                    });
                });

            return deferred.promise();
        },

        _refresh: function(){
            this._init_table();
        },

        _annotate_scores: function(){
            var _this = this;
            var stats = function(l, key){
                var sum = 0;
                for (var i = 0; i < l.length; i++) {
                    sum += l[i][key];
                }
                var m = sum / l.length;
                var v = 0;
                for (var i = 0; i < l.length; i++){
                    v += (l[i][key] - m) ** 2;
                }
                var std = Math.sqrt(v);
                return {'mean': m, 'std': std, 'var': v};
            };

            var surfer2scores = new Map();
            $.each(this.scores, function(idx, score){
                var surfer_id = score['surfer_id'];
                var wave = score['wave'];
                if (!surfer2scores.has(surfer_id)) {
                    surfer2scores.set(surfer_id, new Map());
                }
                var wave2scores = surfer2scores.get(surfer_id);
                if (!wave2scores.has(wave)) {
                    wave2scores.set(wave, [])
                }
                wave2scores.get(wave).push(score);
            });

            var n_judges = _this.judge_assignments.length;
            surfer2scores.forEach(function(wave2scores, surfer_id){
                wave2scores.forEach(function(scores, wave){
                    scores.sort(function(a, b){
                        return a['score'] - b['score'];
                    });
                    var s = stats(scores, 'score');
                    $.each(scores, function(idx, score){
                        if (Math.abs(score['score'] - s['mean']) >= 2) {
                            score['conspicious'] = true;
                        }
                    });

                    if ((n_judges >= 5) && (scores.length == n_judges)) {
                        scores[0]['extremal'] = true;
                        scores[scores.length - 1]['extremal'] = true;
                    }
                });
            });
        },

        _init_table: function(){
            var _this = this;
            var table = this.element.find('table');
            table.empty();
            var header_row = $('<tr>');
            header_row.append($('<td>&nbsp;</td>'));
            header_row.append($('<td><b>Surfer</b></td>'));
            header_row.append($('<td><b>Judge</b></td>'));
            for (var i=0; i < this.heat['number_of_waves']; i++){
                header_row.append($('<td>', {class: 'text-center'})
                                  .attr('colspan', 2)
                                  .append($('<b>')
                                          .html(i + 1)));
            }
            table.append($('<thead>').append(header_row));
            this.heat['participations'].sort(function(a, b){
                return a['seed'] - b['seed'];
            });

            var body = $('<tbody>');
            $.each(this.heat['participations'], function(pidx, participation){
                var participant = participation['surfer'];
                var n_judges = _this.judge_assignments.length;
                $.each(_this.judge_assignments, function(jidx, judge_assignment){
                    var judge = judge_assignment['judge'];
                    var color_str = _this.participant_info.get(participant['id'])['lycra_color']['name'];
                    var color_hex = _this.participant_info.get(participant['id'])['lycra_color']['hex'];
                    var row = $('<tr>', {
                        style: 'background-color:' + _this._make_lighter(color_hex) + ';', // the last digits are the opacity in hex
                    });

                    // column for strong lycra color
                    row.append($('<td>', {
                        style: 'background-color:' + color_hex + ';',
                    }));

                    // fill surfer name cell
                    row.append($('<td>')
                               .html(participant['first_name'] + ' ' + participant['last_name']));

                    // fill judge name cell
                    row.append($('<td>')
                               .html(judge['first_name'] + ' ' + judge['last_name']));

                    // filter and sort scores for surfer
                    var scores = _this._get_scores_for_surfer(judge['id'], participant['id']);

                    // fill table cell with score
                    for (var i = 0; i < _this.heat['number_of_waves']; i++){
                        var score_val = '';
                        var classes = 'score_elem';
                        if (i < scores.length){
                            var score = scores[i];
                            if (score['interference'])
                                score_val = 'I';
                            else if (score['missed'])
                                score_val = 'M';
                            else
                                score_val = score['score'].toFixed(1);

                            if (_this.options.mark_conspicious && score['conspicious'])
                                classes += ' conspicious';
                            if (_this.options.mark_extremal && score['extremal'])
                                classes += ' extremal';
                        }
                        if (i <= scores.length)
                            classes += ' editable';
                        if (i == scores.length - 1)
                            classes += ' deletable';

                        row.append($('<td>', {
                            class: classes,
                            data: {judge_id: judge['id'], surfer_id: participant['id'], wave: i, color_hex: color_hex},
                        })
                                   .html(score_val));
                        if (jidx == 0) {
                            var wave_score = _this.prelim_results.get(participant['id'])['wave_scores'][i] || {};
                            var val = wave_score['score'];
                            if (val == null) {
                                val = '';
                            } else {
                                val = val.toFixed(2);
                            }

                            // combined score
                            row.append($('<td>', {
                                html: val,
                                class: 'combined_score text-center',
                            })
                                .attr('rowspan', n_judges));
                        }
                    }
                    // add row to table body
                    body.append(row);
                });
            });
            table.append(body);
        },

        _get_scores_for_surfer: function(judge_id, surfer_id){
            return this.scores
                .filter(function(score){
                    return (score['surfer_id'] == surfer_id && score['judge_id'] == judge_id)
                })
                .sort(function(s1, s2){
                    return s1['wave'] - s2['wave'];
                });
        },

        _register_events: function(){
            this._on(this.element, {
                'click .score_elem': function(ev){
                    var elem = $(ev.currentTarget);
                    if (elem.hasClass('editable')){
                        var deletable = elem.hasClass('deletable');
                        var data = elem.data();
                        this._init_edit_score_modal(data['judge_id'], data['surfer_id'], data['wave'], this._make_lighter(data['color_hex']), deletable);
                    }
                },
            })
        },

        _init_edit_score_modal: function(judge_id, surfer_id, wave, color_hex, deletable){
            var _this = this;
            var participant_scores = this._get_scores_for_surfer(judge_id, surfer_id);
            if (wave > participant_scores.length)
                return;

            var bb = bootbox.dialog({
                closeButton: false,
                message: '<div class="edit_score"></div>',
                size: 'large',
            });
            bb.init(function(){
                var edit_score_elem = bb.find('.edit_score');
                edit_score_elem.edit_score({
                    heat_id: _this.options.heat_id,
                    judge_id: judge_id,
                    surfer_id: surfer_id,
                    wave: wave,
                    old_score: participant_scores[wave] || null,
                    background_color: color_hex,
                    delete_allowed: deletable,
                });

                edit_score_elem.on('edit_scorecancelled', function(){
                    bootbox.hideAll();
                    _this.refresh();
                });
                edit_score_elem.on('edit_scoresubmitted', function(){
                    bootbox.hideAll();
                    _this.refresh();
                });
            });
        },

        _make_lighter: function(hex) {
            return lighten(hex);
        },
    });
}(jQuery));
