/* =========================================================
 * edit_surfer.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_surfer', {
        options: {
            surfer_id: null,
            data: null,
            geturl: 'rest/surfers/{surferid}',
            posturl: '/rest/surfers/{surferid}',
            deleteurl: '/rest/surfers/{surferid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_surfer module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.surfer_id !== null)
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
                '  <div class="form-group row">',
                '    <label class="col-2 col-form-label">Name</label>',
                '    <div class="col-5">',
                '        <input type="hidden" data-key="id" class="surfer_input" id="id">',
                '        <input type="text" data-key="first_name" name="first_name" class="form-control surfer_input" placeholder="First Name">',
                '     </div>',
                '     <div class="col-5">',
                '        <input type="text" data-key="last_name" name="last_name" class="form-control surfer_input" placeholder="Last Name">',
                '     </div>',
                '  </div>',
                '  <div class="form-group row">',
                '     <label class="col-2 col-form-label">Country</label>',
                '     <div class="col-10">',
                '         <input type="text" data-key="country" name="country" class="form-control surfer_input" placeholder="Country">',
                '         </div>',
                '     </div>',
                '     <div class="form-group row">',
                '         <label class="col-2 col-form-label">Additional Info</label>',
                '         <div class="col-10">',
                '             <input type="text" data-key="additional_info" name="additional_info" class="form-control surfer_input" placeholder="Additional Info">',
                '         </div>',
                '     </div>',
                '  </div>',
                '  <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '  <div class="float-right">',
                '    <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '    <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '  </div>',
                '</div>',
                '</form>',
            ].join(' ');

            this.element.append(html);

            if (this.options.surfer_id == null) {
                this.element.find('.delete_btn').remove();
            }
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_surfer,
                'click .save_changes_btn': this.upload,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.surfer_id !== null){
                $.getJSON(this.options.geturl.format({surferid: this.options.surfer_id}))
                    .done(function(surfer){
                        if (surfer != null){
                            _this.data = surfer;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('Surfer not found.');
                            _this.options.surfer_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no surfer id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('.surfer_input').each(function(idx, elem){
                var key = $(this).data('key');
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
            var post_surfer_id = this.options.surfer_id || 'new';
            $.post(this.options.posturl.format({surferid: post_surfer_id}), JSON.stringify(this.data), function(surfer){
                _this.options.surfer_id = surfer['id'];
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

        delete_surfer: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.surfer_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({surferid: this.options.surfer_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    // initialize module as empty
                    _this.options.surfer_id = _this.options.surfer_id || _this.data['surfer_id'];
                    _this.options.surfer_id = null;
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
                console.log('No surfer id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            this.element.find('.surfer_input').each(function(idx, elem){
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
