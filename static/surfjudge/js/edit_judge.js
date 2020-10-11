/* =========================================================
 * edit_judge.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_judge', {
        options: {
            judge_id: null,
            data: null,
            geturl: 'rest/judges/{judgeid}',
            posturl: '/rest/judges/{judgeid}',
            deleteurl: '/rest/judges/{judgeid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_judge module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.judge_id !== null)
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
                '<div class="form-group row">',
                '    <label class="col-2 col-form-label">Judge name</label>',
                '    <div class="col-5">',
                '        <input type="text" data-key="first_name" name="first_name" class="form-control judge_input" placeholder="First Name">',
                '    </div>',
                '    <div class="col-5">',
                '        <input type="text" data-key="last_name" name="last_name" class="form-control judge_input" placeholder="Last Name">',
                '    </div>',
                '</div>',
                '<div class="form-group row">',
                '    <label class="col-2 col-form-label">Login</label>',
                '    <div class="col-10">',
                '        <input type="text" data-key="username" name="username" class="form-control judge_input">',
                '    </div>',
                '</div>',
                '',,
                '<div class="form-group row">',
                '    <label class="col-2 col-form-label">Additional Info</label>',
                '    <div class="col-10">',
                '        <input type="text" data-key="additional_info" name="additional_info" class="form-control judge_input" placeholder="Additional Info">',
                '    </div>',
                '</div>',
                '  <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '  <div class="float-right">',
                '    <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '    <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '  </div>',
                '</form>',
            ].join(' ');

            this.element.append(html);

            if (this.options.judge_id == null) {
                this.element.find('.delete_btn').remove();
            }
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_judge,
                'click .save_changes_btn': this.upload,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.judge_id !== null){
                $.getJSON(this.options.geturl.format({judgeid: this.options.judge_id}))
                    .done(function(judge){
                        if (judge != null){
                            _this.data = judge;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('Judge not found.');
                            _this.options.judge_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no judge id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('.judge_input').each(function(idx, elem){
                var key = $(elem).data('key');
                $(this).val(_this.data[key]);
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
            var post_judge_id = this.options.judge_id || 'new';
            $.post(this.options.posturl.format({judgeid: post_judge_id}), JSON.stringify(this.data), function(judge){
                _this.options.judge_id = judge['id'];
                _this.refresh().done(function(){
                    _this._trigger('data_changed', null);
                });
                deferred.resolve();
            })
                .fail(function(ev){
                    console.log('Connection error.');
                });
            return deferred.promise();
        },

        delete_judge: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.judge_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({judgeid: this.options.judge_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    // initialize module as empty
                    _this.options.judge_id = _this.options.judge_id || _this.data['judge_id'];
                    _this.options.judge_id = null;
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
                console.log('No judge id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            this.element.find('.judge_input').each(function(idx, elem){
                _this.data[$(this).data('key')] = $(this).val();
            });
        },

        _check_data: function(){
            if (this.data['first_name'].length == 0) {
                alert('Empty field "First name"');
                return false;
            }
            if (this.data['last_name'].length == 0) {
                alert('Empty field "Last name"');
                return false;
            }
            if (this.data['username'].length == 0) {
                alert('Empty field "Login"');
                return false;
            }
            return true;
        },

        _mark_dirty: function(){
            this.element.find('.dirty_marker').addClass('alert-danger');
        },

        _mark_clean: function(){
            this.element.find('.dirty_marker').removeClass('alert-danger');
        },
    });
}(jQuery));
