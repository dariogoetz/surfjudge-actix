/* =========================================================
 * edit_tournament.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_tournament', {
        options: {
            tournament_id: null,
            data: null,
            geturl: 'rest/tournaments/{tournamentid}',
            posturl: '/rest/tournaments/{tournamentid}',
            deleteurl: '/rest/tournaments/{tournamentid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_tournament module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.tournament_id !== null)
                this._initialized = this.refresh();
            else
                this._initialized = $.Deferred().resolve().promise();
        },

        _destroy: function(){
            this.element.empty();
        },

        _check_inputs: function(){
            return true;
        },

        _init_html: function(){
            var _this = this;
            var html = [
                '<form>',
                '<div class="alert dirty_marker">',
                '    <div class="form-group row">',
                '        <label class="col-3 control-label">Tournament Name</label>',
                '        <div class="col-9">',
                '            <input type="hidden" class="tournament_input" data-key="id">',
                '            <input type="text" name="name" class="form-control tournament_input" data-key="name" placeholder="Tournament Name">',
                '        </div>',
                '    </div>',
                '    <div class="form-group row">',
                '        <label class="col-3 control-label">Dates</label>',
                '        <div class="col-5 input-group date start_date">',
                '            <input class="form-control tournament_input" type="text" placeholder="Date start" data-key="start_date" readonly>',
                '            <span class="input-group-addon add-on"><i class="fa fa-calendar"></i></span>',
                '        </div>',
                '        <div class="col-4 input-group date end_date">',
                '           <input class="form-control tournament_input" type="text" placeholder="Date end" data-key="end_date" readonly>',
                '           <span class="input-group-addon add-on"><i class="fa fa-calendar"></i></span>',
                '        </div>',
                '    </div>',
                '    <div class="form-group row">',
                '        <label class="col-3 control-label">Additional Info</label>',
                '        <div class="col-9">',
                '            <input type="text" name="additional_info" class="form-control tournament_input" data-key="additional_info" placeholder="Additional Info">',
                '        </div>',
                '    </div>',
                '    <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '    <div class="float-right">',
                '        <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '        <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '    </div>',
                '    <hr>',
                '    <div class="form-group row">',
                '        <button type="button" class="btn btn-success export_btn">Export Results</button>',
                '    </div>',
                '</div>',
                '</form>',
            ].join(' ');

            this.element.append(html);

            if (this.options.tournament_id == null) {
                this.element.find('.delete_btn').remove();
            }

            this.element.find('.date').datepicker({
                weekStart: 1,
                autoclose: true,
                format: 'yyyy-mm-dd',
                todayHighlight: true,
            });
            this.element.find('.date.start_date').datepicker('setDate', this.data['start_date'] || new Date());
            this.element.find('.date.end_date').datepicker('setDate', this.data['end_date'] || new Date());
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_tournament,
                'click .save_changes_btn': this.upload,
                'click .export_btn': this.export_scores,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.tournament_id !== null){
                $.getJSON(this.options.geturl.format({tournamentid: this.options.tournament_id}))
                    .done(function(tournament){
                        if (tournament != null){
                            _this.data = tournament;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('Tournament not found.');
                            _this.options.tournament_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no tournament id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('.tournament_input').each(function(idx, elem){
                var key = $(this).data('key');
                if (key == 'start_date') {
                    _this.element.find('.date.start_date').datepicker('setDate', _this.data[key] || new Date());
                } else if (key == 'end_date') {
                    _this.element.find('.date.end_date').datepicker('setDate', _this.data[key] || new Date());
                }
                else {
                    $(this).val(_this.data[key]);
                }
            });
        },

        upload: function(){
            var _this = this;
            this._fetch_details_from_inputs();
            if (!this._check_data()){
                console.log('Upload failed due to data check');
                return;
            }

            console.log('Uploading');
            var deferred = $.Deferred();
            var post_tournament_id = this.options.tournament_id || 'new';
            $.post(this.options.posturl.format({tournamentid: post_tournament_id}), JSON.stringify(this.data), function(tournament){
                _this.options.tournament_id = tournament['id'];
                _this.refresh().done(function(){
                    _this._trigger('data_changed', null);
                    deferred.resolve();
                });
            })
                .fail(function(ev){
                    console.log('Connection error.');
                });
            return deferred.promise();
        },

        delete_tournament: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.tournament_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({tournamentid: this.options.tournament_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    // initialize module as empty
                    _this.options.tournament_id = _this.options.tournament_id || _this.data['id'];
                    _this.options.tournament_id = null;
                    _this.data = {};
                    _this._refresh();
                    _this._trigger('deleted');
                    _this._trigger('data_changed');
                    deferred.resolve();
                })
                .fail(function(){
                    console.log('Connection error.');
                    deferred.reject();
                });
            } else {
                console.log('No tournament id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        export_scores: function(){
            window.location.replace('/rest/export_results_for_tournament/' + this.options.tournament_id);
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            this.element.find('.tournament_input').each(function(idx, elem){
                _this.data[$(this).data('key')] = $(this).val();
            });
        },

        _check_data: function(){
            if (this.data['name'].length == 0) {
                alert('Empty field "Tournament Name"');
                return false;
            }
            var start_date = this.element.find('.date.start_date').datepicker('getDate');
            var end_date = this.element.find('.date.end_date').datepicker('getDate');
            if (start_date > end_date) {
                alert("Start date after end date");
                return false;
            }
            return true;
        },

        _mark_dirty: function(){
            if (this.options.tournament_id == null) {
                // no dirty marking for new tournament
                return;
            }
            this.element.find('.dirty_marker').addClass('alert-danger');
        },

        _mark_clean: function(){
            this.element.find('.dirty_marker').removeClass('alert-danger');
        },
    });
}(jQuery));
