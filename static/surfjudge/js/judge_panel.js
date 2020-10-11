(function($, undefined){
    $.widget('surfjudge.judge_panel', {
        options: {
            heat_id: null,
            judge_id: null,
            heat_data: null,
            score_data: null,

            getheaturl: '/rest/heats/{heatid}',
            getscoresurl: '/rest/scores',

            websocket_url: null,
        },

        _create: function(){
            this._init_html();

            this._scores_by_surfer = null;

            console.log('Initiating websocket for judge panel.')

            var on_score_msg = function(msg){
                var msg = JSON.parse(msg);
                var heat_id = msg["heat_id"];
                var judge_id = msg["judge_id"];
                if ((heat_id == this.options.heat_id) && (judge_id == this.options.judge_id)) {
                    console.log("My scores changed: refreshing");
                    this.refresh();
                } else {
                    console.log("Other scores changed: not refreshing");
                }
            }
            this.websocket = new WebSocketClient({
                url: this.options.websocket_url,
                channels: {
                    'scores': on_score_msg.bind(this),
                },
                name: 'Judge Panel',
            });

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
            html = $([
                '<table class="table table-bordered judging_table">',
                '</table>',
            ].join(' '));

            this.element.append(html);
        },

        refresh: function(){

            var _this = this;
            var deferred_heat = $.getJSON(this.options.getheaturl.format({heatid: this.options.heat_id}));
            var deferred_scores = $.getJSON(this.options.getscoresurl, {judge_id: this.options.judge_id, heat_id: this.options.heat_id});

            var deferred = $.Deferred();
            $.when(deferred_heat, deferred_scores).done(function(heat_data, score_data){
                _this.heat_data = heat_data[0];
                _this.score_data = score_data[0];

                _this._refresh();
                deferred.resolve();
            })
                .fail(function(){
                    console.log('Connection error');
                    deferred.reject();
                });
            return deferred.promise();
        },

        _refresh: function(){
            this._compute_scores_by_surfer();
            this._init_judging_table();
        },

        _compute_scores_by_surfer: function(){
            var _this = this;
            var tmp = new Map();
            $.each(this.score_data, function(idx, score){
                var surfer_id = score['surfer_id'];
                if (!tmp.has(surfer_id)) {
                    // prepare scores list if not yet existing
                    tmp.set(surfer_id, []);
                }
                // add score to surfers scores list
                tmp.get(score['surfer_id']).push(score);
            });
            // sort scores lists
            this._scores_by_surfer = new Map();
            tmp.forEach(function(scores, surfer_id){
                _this._scores_by_surfer.set(surfer_id, scores.sort(function(a,b){
                    return a['wave'] - b['wave'];
                }));
            });
        },

        _register_events: function(){
            this._on(this.element, {
                'click .init_score_entry': function(ev){
                    var data = $(ev.currentTarget).data();
                    this._init_edit_score_modal(data['id'], data['wave'], this._make_lighter(data['color_hex']));
                },
            });
        },

        _init_judging_table: function(){
            var _this = this;
            var n_waves = this.heat_data['number_of_waves'];
            var table_elem = this.element.find('.judging_table');
            table_elem.empty();

            var best_waves = this._get_best_waves();

            // table header
            var table_header = $('<thead class="thead-dark">');
            var header_row = $('<tr>');
            header_row.append('<td><b>Wave</b></td>');
            for (var i = 0; i < n_waves; i++){
                header_row.append($('<td>').text(i+1));
            }
            table_header.append(header_row);
            table_elem.append(table_header)

            table_body = $('<tbody>');
            // table body
            this.heat_data['participations'].sort(function(a, b){
                return a['seed'] - b['seed'];
            });
            $.each(this.heat_data['participations'], function(idx, participation){
                var trow = $('<tr>');
                // surfer identifier (first column)
                var color_name = participation['lycra_color']['name'];
                color_name = color_name.charAt(0).toUpperCase() + color_name.slice(1)
                var bg_color_hex = lighten_darken_color(participation['lycra_color']['hex'], 100);
                var index_elem = $('<td>', {
                    class: "color_elem init_score_entry",
                    style: "background-color: " + bg_color_hex + '; color: ' + text_color_for_background(bg_color_hex),
                    data: {id: participation['surfer_id'], wave: -1, color_hex: participation['lycra_color']['hex']}
                })
                    .append($('<b>').text(color_name));

                // add index column to row
                trow.append(index_elem);

                // individual waves
                for (var wave = 0; wave < n_waves; wave++){
                    // generate score element
                    var score_cell = $('<td>').addClass('score_elem init_score_entry');
                    score_cell.data({id: participation['surfer_id'], wave: wave, color_hex: participation['lycra_color']['hex']});

                    // mark best / inactive
                    var participation_scores = _this._scores_by_surfer.get(participation['surfer_id']) || [];
                    if (wave == best_waves[participation['surfer_id']]['wave'])
                        score_cell.addClass('best_wave');
                    else if (wave > participation_scores.length)
                        score_cell.addClass('inactive');
                    else if (wave == participation_scores.length)
                        score_cell.addClass('next_wave');

                    if (wave <= participation_scores.length)
                        score_cell.addClass('init_score_entry');

                    // insert score value
                    var score_val = $([
                        '<div class="text-center">'
                    ].join(' '))
                    var score = participation_scores[wave] || {};
                    if (score['missed'])
                        score_val.text('M');
                    else if (score['interference'])
                        score_val.text('M');
                    else if ('score' in score)
                        score_val.text(score['score'].toFixed(1));
                    // add column to row
                    trow.append(score_cell.append(score_val));
                }
                table_body.append(trow);
            });
            table_elem.append(table_body);
        },

        _get_best_waves: function(){
            var best_waves = {};
            $.each(this.heat_data['participations'], function(idx, participation){
                best_waves[participation['surfer_id']] = {};
            });
            this._scores_by_surfer.forEach(function(scores, surfer_id){
                var best_wave = {};
                $.each(scores, function(idx, score){
                    if (!score['missed'] && !score['interference'])
                        if (score['score'] > (best_wave['score'] || -1))
                            best_wave = score;
                });
                best_waves[surfer_id] = best_wave;
            });
            return best_waves;
        },


        _init_edit_score_modal: function(surfer_id, wave, color_hex){
            var _this = this;
            var participation_scores = this._scores_by_surfer.get(surfer_id) || [];
            if (wave < 0)
                wave = participation_scores.length;

            if (wave > participation_scores.length)
                return;

            var bb = bootbox.dialog({
                onEscape: true,
                closeButton: false,
                message: '<div class="edit_score"></div>',
                size: 'large',
            });
            bb.init(function(){
                var edit_score_elem = bb.find('.edit_score');
                edit_score_elem.edit_score({
                    heat_id: _this.options.heat_id,
                    judge_id: _this.options.judge_id,
                    surfer_id: surfer_id,
                    wave: wave,
                    old_score: participation_scores[wave] || null,
                    background_color: color_hex,
                });

                edit_score_elem.on('edit_scorecancelled', function(){
                    bootbox.hideAll();
                    // a refresh is not necessary here, because nothing changed
                    // we do it here again anyways for safety
                    _this.refresh();
                });
                edit_score_elem.on('edit_scoresubmitted', function(){
                    bootbox.hideAll();
                    // refresh is also called via websocket
                    // but if websockets are not working, for some reason
                    // we do it here again for safety
                    _this.refresh();
                });
            });
        },

        _make_lighter: function(hex){
            return lighten(hex);
        },
    });

}(jQuery));
