/* =========================================================
 * generate_category.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */
(function($, undefined){

    $.widget('surfjudge.generate_heats', {
        options: {
            category_id: null,
            postheaturl: '/rest/heats',
            postadvancementsurl: '/rest/advancements',
            postsurferurl: '/rest/surfers',
            putparticipantsurl: '/rest/participants/{heatid}',
            getlycracolorsurl: '/rest/lycra_colors',
            heatchart_width: 520,
            margin_left: 0,
            margin_right: 0,
            margin_top: 0,
            margin_bottom: 0,

            heat_types:  [
                {key: 'standard', label: 'Standard'},
                {key: 'rsl', label: 'Rapid Surf League'},
                {key: 'custom', label: 'Custom'},
            ],
        },

        _create: function(){
            var _this = this;
            this.use_absolute_seeds = false;
            this.generator = null;
            this.generator_type = null;

            this.lycra_colors = null; // used for preview in Tournamentgenerator

            var deferred = $.Deferred();
            $.get(this.options.getlycracolorsurl)
                .done(function(lycra_colors){
                    _this.lycra_colors = lycra_colors;
                    deferred.resolve();
                })
                .fail(function(){
                    console.error('Could not retrieve lycra colors.');
                    deferred.resolve();
                });
            this.deferred_lycra_colors = deferred.promise();

            this._init_html();
            this._register_events();
        },

        _destroy: function(){
            this.element.empty();
        },

        _init_html: function(){
           var _this = this;
           var html = $([
               '<div class="form-horizontal">',
               '  <div class="form-group">',
               '    <div class="row heat_options">',
               '      <label class="col-4">Number of waves</label>',
               '      <div class="col-4">',
               '        <div class="input-group plusminusinput">',
               '          <span class="input-group-btn">',
               '            <button type="button" class="btn btn-danger btn-number" data-type="minus" data-field="nwaves">',
               '              <span class="fa fa-minus"></span>',
               '            </button>',
               '          </span>',
               '          <input type="text" name="nwaves" class="form-control input-number nwaves" data-key="nwaves" placeholder="10" min="1" max="100" value="10">',
               '          <span class="input-group-btn">',
               '            <button type="button" class="btn btn-success btn-number" data-type="plus" data-field="nwaves">',
               '              <span class="fa fa-plus"></span>',
               '            </button>',
               '          </span>',
               '        </div>',
               '      </div>',
               '    </div>',
               '    <br>',
               '    <div class="row heat_options">',
               '      <label class="col-4">Duration [min]</label>',
               '      <div class="col-4">',
               '        <div class="input-group plusminusinput">',
               '          <span class="input-group-btn">',
               '            <button type="button" class="btn btn-danger btn-number" data-type="minus" data-field="duration">',
               '              <span class="fa fa-minus"></span>',
               '            </button>',
               '          </span>',
               '          <input type="text" name="duration" class="form-control input-number duration" data-key="duration" placeholder="15" min="0" max="100" value="15">',
               '          <span class="input-group-btn">',
               '            <button type="button" class="btn btn-success btn-number" data-type="plus" data-field="duration">',
               '              <span class="fa fa-plus"></span>',
               '            </button>',
               '          </span>',
               '        </div>',
               '      </div>',
               '    </div>',
               '    <br>',
               '    <div class="row">',
               '      <label class="col-4">Type</label>',
               '      <div class="col-4">',
               '        <select class="form-control heat_type_select" data-field="heat_type"></select>',
               '      </div>',
               '      <div class="col-4">',
               '        <div class="float-right">',
               '          <button type="button" class="btn btn-secondary upload_csv">Load CSV</button>',
               '          <div class="form-check">',
               '            <input type="checkbox" class="form-check-input use_absolute_seeds">',
               '            <label class="form-check-label">Use absolute seeds</label>',
               '          </div>',
               '        </div>',
               '      </div>',
               '    </div>',
               '    <div class="heatchart_options">',
               '    </div>',
               '  </div>',
               '</div>',
               '<h4>Preview</h4>',
               '<div class="preview_heatchart">',
               '</div>',
               '<div class="float-right">',
               '  <button type="button" class="btn btn-secondary clear_csv" disabled>Reset</button>',
               '  <button type="button" class="btn btn-primary upload_btn" disabled>Save Changes</button>',
               '</div>',

            ].join(' '));
            html.find('.plusminusinput').plusminusinput();

            this.element.append(html);

            var type_menu = this.element.find('.heat_type_select');
            type_menu.empty();
            $.each(this.options.heat_types, function(idx, type_pair){
                type_menu.append('<option data-value="{0}">{1}</option>'.format(type_pair['key'], type_pair['label']));
            });
            var options = {};
            this._select_heatchart_type();
            this.generate_chart();
        },

        _register_events: function(){
            this._on(this.element, {
                'click .upload_btn': this.upload,
                'change .heatchart_options': this.generate_chart,
                'change .heat_options': this._set_generator_options,
                'click .upload_csv': this._show_upload_csv_widget,
                'click .clear_csv': this._clear_csv_data,
                'change .use_absolute_seeds': this._toggle_use_absolute,
                'change .heat_type_select': this._select_heatchart_type,
            });
        },

        _toggle_use_absolute: function() {
            this.use_absolute_seeds = this.element.find('.use_absolute_seeds').is(":checked");
            this.generate_chart();
        },

        _clear_csv_data: function(){
            var $btn = this.element.find('.clear_csv');
            if ($btn.attr('disabled')) {
                return;
            } else {
                this.element.find('.upload_csv').data('participants', null);
                $btn.attr('disabled', true);
                this.generate_chart();
            }
        },

        _show_upload_csv_widget: function() {
            var _this = this;
            var html = $([
                '<div class="upload_widget"></div>',
            ].join(' '));
            var bb = bootbox.dialog({
                title: 'Upload Category Participants',
                size: 'large',
                message: html,
                onEscape: true,
                buttons: {
                    'cancel': {
                        label: 'Cancel',
                        className: 'btn btn-default',
                    }
                }
            });

            bb.init(function(){
                bb.find('.upload_widget').csv_upload({
                    required_columns: ['seed', 'first_name', 'last_name'],
                    expected_columns: ['seed', 'first_name', 'last_name'],
                    delimiter: "", // auto
                });
                bb.find('.upload_widget').on('csv_uploaddata_changed', function(ev, data, b, c){
                    // feed upload_data to heatchart / tournament_generator
                    data['data'].sort(function(a, b) {
                        return a['seed'] - b['seed'];
                    });
                    _this.element.find('.upload_csv').data('participants', data['data']);
                    _this.generate_chart();
                    _this.element.find('.clear_csv').attr('disabled', false);
                    bb.modal('hide');
                });
            });
        },

        _set_generator_options: function() {
            this.generator.options["number_of_waves"] = this.element.find('.input-number.nwaves').val();
            this.generator.options["duration"] = this.element.find('.input-number.duration').val();
        },

        _select_heatchart_type: function() {
            var _this = this;
            this.generator_type = this.element.find('.heat_type_select > option:selected').data('value');
            this.deferred_lycra_colors.done(function(){
                var generator_options = {
                    postheaturl: _this.options.postheaturl,
                    postadvancementsurl: _this.options.postadvancementsurl,
                    postsurferurl: _this.options.postsurferurl,
                    putparticipantsurl: _this.options.putparticipantsurl,
                    getlycracolorsurl: _this.options.getlycracolorsurl,
                    preview_lycra_colors: _this.lycra_colors,

                };

                if (_this.generator_type == 'standard') {
                    _this.generator = new StandardTournamentGenerator(generator_options)
                } else if (_this.generator_type == 'rsl') {
                    _this.generator = new RSLTournamentGenerator(generator_options);
                } else {
                    _this.generator = new CustomTournamentGenerator(generator_options);
                }
                _this._set_generator_options();
                _this._set_heatchart_options_html();
                _this.generate_chart();
            });
        },

        _set_heatchart_options_html: function(){
            var html = $("<span> TBD </span>");
            if (this.generator_type == 'standard' || this.generator_type == 'rsl') {
                html = $([
                    '<div class="row">',
                    '  <label class="col-4">Number of rounds</label>',
                    '  <div class="col-4">',
                    '    <div class="input-group plusminusinput">',
                    '      <span class="input-group-btn">',
                    '        <button type="button" class="btn btn-danger btn-number" data-type="minus" data-field="nheats">',
                    '          <span class="fa fa-minus"></span>',
                    '        </button>',
                    '      </span>',
                    '      <input type="text" name="nheats" class="form-control input-number nheats" data-key="nheats" placeholder="3" min="1" max="100" value="2">',
                    '      <span class="input-group-btn">',
                    '        <button type="button" class="btn btn-success btn-number" data-type="plus" data-field="nheats">',
                    '          <span class="fa fa-plus"></span>',
                    '        </button>',
                    '      </span>',
                    '    </div>',
                    '  </div>',
                    '</div>',
                ].join(' '));

                // ***** plusminus buttons *****
                html.find('.plusminusinput').plusminusinput();
            } else {
                html = $([
                    '<div class="row">',
                    '  <label class="col-4">Number of rounds</label>',
                    '  <div class="col-4">',
                    '    <div class="input-group plusminusinput">',
                    '      <span class="input-group-btn">',
                    '        <button type="button" class="btn btn-danger btn-number" data-type="minus" data-field="nheats">',
                    '          <span class="fa fa-minus"></span>',
                    '        </button>',
                    '      </span>',
                    '      <input type="text" name="nheats" class="form-control input-number nheats" data-key="nheats" placeholder="3" min="1" max="100" value="2">',
                    '      <span class="input-group-btn">',
                    '        <button type="button" class="btn btn-success btn-number" data-type="plus" data-field="nheats">',
                    '          <span class="fa fa-plus"></span>',
                    '        </button>',
                    '      </span>',
                    '    </div>',
                    '  </div>',
                    '</div>',
                    '<div>',
                    '  <div class="round_options">',
                    '  </div>',
                    '</div>',
                ].join(' '));

                var add_round_options = function(){
                    var rounds = html.find('input.input-number.nheats').val();
                    var round_options_elem = html.find('.round_options');
                    round_options_elem.empty();
                    round_options_elem.append($('<br>'));
                    for (var round = 0; round < rounds; round++) {
                        var round_name = 'Final';
                        if (round == 1) {
                            round_name = 'Semi-Final {number}';
                        } else if (round > 1) {
                            round_name = "Round " + (rounds - round) + " Heat {number}";
                        }
                        var nheats = round + 1;

                        var row = $('<div>', {class: 'row round', data: {round: round}});
                        row.append($('<label>', {text: "Heat Name Template", class: "col-3"}))
                        row.append($([
                            '<div class="col-4">',
                            '  <input type="text" data-key="name" name="round_name" class="form-control" placeholder="Round Name" value="{0}">'.format(round_name),
                            '</div>',
                        ].join('')))
                        row.append($('<label>', {text: "Number of Heats", class: "col-3"}))
                        row.append($([
                            '<div class="col-2">',
                            '  <input type="number" data-key="heats_in_round" name="heats_in_round", class="form-control" placeholder="Number of Heats in Round" value="{0}">'.format(nheats),
                            '</div>',
                        ].join('')));
                        round_options_elem.append(row);
                    }
                }

                html.find('input.input-number.nheats').on('change', add_round_options);

                add_round_options();

                // ***** plusminus buttons *****
                html.find('.plusminusinput').plusminusinput();
            }
            this.element.find('.heatchart_options').empty().append(html);
        },

        _get_heatchart_options: function(){
            var chart_options;
            if (this.generator_type == 'standard' || this.generator_type == 'rsl') {
                chart_options = this.element.find('.heatchart_options .input-number.nheats').val();
            } else {
                chart_options = [];
                this.element.find('.heatchart_options div.round').each(function(idx, elem){
                    var d = {};
                    d['round'] = $(this).data('round');
                    $(this).find('input').each(function(idx, elem){
                        var key = $(elem).data('key');
                        d[key] = $(elem).val();
                    });
                    chart_options.push(d);
                });
            }
            return chart_options;
        },

        generate_chart: function(){
            var _this = this;
            this.deferred_lycra_colors.done(function(){
                _this._generate_chart();
            });
        },

        _generate_chart: function(){
            var participant_data = this.element.find('.upload_csv').data('participants');
            var chart_options = this._get_heatchart_options();
            console.log(chart_options);

            var heatchart_data = this.generator.generate_heatchart_data(chart_options, participant_data, !this.use_absolute_seeds);

            var heatchart_elem = this.element.find('.preview_heatchart');
            if (heatchart_elem.data('surfjudgeHeatchart') != null)
                heatchart_elem.heatchart('destroy');
            heatchart_data['websocket_url'] = null;
            heatchart_data['use_websocket'] = false;
            heatchart_data['width'] = this.options.heatchart_width;
            heatchart_data['margin_left'] = this.options['margin_left'];
            heatchart_data['margin_right'] = this.options['margin_right'];
            heatchart_data['margin_top'] = this.options['margin_top'];
            heatchart_data['margin_bottom'] = this.options['margin_bottom'];
            heatchart_data['show_individual_scores'] = false,
            heatchart_data['show_total_scores'] = false,
            heatchart_elem.heatchart(heatchart_data);

            this._activate_upload_btn();
        },

        _activate_upload_btn: function(){
            if (this.generator.heatchart_data != null)
                this.element.find('.upload_btn').attr('disabled', false);
            else
                this.element.find('.upload_btn').attr('disabled', true);
        },

        upload: function(){
            var _this = this;
            if (this.generator.heatchart_data != null && this.options.category_id != null){
                var deferred = this.generator.upload(this.options.category_id);
                deferred.done(function(){
                    _this._trigger('data_changed');
                });
            } else {
            }
        },
    });
}(jQuery));
