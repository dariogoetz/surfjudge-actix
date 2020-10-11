/* =========================================================
 * edit_category.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_category', {
        options: {
            category_id: null,
            tournament_id: null,
            data: null,
            geturl: 'rest/categories/{categoryid}',
            posturl: '/rest/categories/{categoryid}',
            deleteurl: '/rest/categories/{categoryid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_category module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.category_id !== null)
                this._initialized = this.refresh();
            else
                this._initialized = $.Deferred().resolve().promise();
        },

        _destroy: function(){
            this.element.empty();
        },

        _check_inputs: function(){
            if (this.options.category_id === null
                && this.options.tournament_id === null)
                // new category, but tournament unspecified
                return false;
            return true;
        },

        _init_html: function(){
            var _this = this;
            var html = [
                '<form>',
                '<div class="alert dirty_marker">',
                '    <div class="form-group row">',
                '        <label class="col-2 control-label">Category Name</label>',
                '        <div class="col-10">',
                '            <input type="hidden" class="category_input" data-key="id">',
                '            <input type="text" name="name" class="form-control category_input" data-key="name" placeholder="Category Name">',
                '        </div>',
                '    </div>',
                '    <div class="form-group row">',
                '        <label class="col-sm-2 control-label">Additional Info</label>',
                '        <div class="col-sm-10">',
                '            <input type="text" name="modal_additional_info" class="form-control category_input" data-key="additional_info" placeholder="Additional Info">',
                '        </div>',
                '    </div>',
                '    <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '    <div class="float-right">',
                '        <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '        <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '    </div>',
                '</div>',
                '</form>',
            ].join(' ');

            this.element.append(html);

            if (this.options.category_id == null) {
                this.element.find('.delete_btn').remove();
            }
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_category,
                'click .save_changes_btn': this.upload,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            var query = {}
            if (this.options.category_id !== null){
                query['category_ids'] = [this.options.category_id];
                $.getJSON(this.options.geturl.format({categoryid: this.options.category_id}))
                    .done(function(category){
                        if (category != null){
                            _this.data = category;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('Category not found.');
                            _this.options.category_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no category id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }

            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;

            this.element.find('.category_input').each(function(idx, elem){
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
            if (this.options.category_id === null){
                this.data['tournament_id'] = this.options.tournament_id;
            }

            var deferred = $.Deferred();
            var post_category_id = this.options.category_id || 'new';
            $.post(this.options.posturl.format({categoryid: post_category_id}), JSON.stringify(this.data), function(category){
                _this.options.category_id = category['id'];
                _this.refresh().done(function(){
                    _this._trigger('data_changed', null);
                    deferred.resolve();
                });
            });
            return deferred.promise();
        },

        delete_category: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.category_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({categoryid: this.options.category_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    console.log("deleted category " + _this.options.category_id);
                    // initialize module as empty
                    _this.options.tournament_id = _this.options.tournament_id || _this.data['tournament_id'];
                    _this.options.category_id = null;
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
                console.log('No category id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        get_data: function(){
            this._fetch_details_from_inputs();
            console.log(this.data);
            return $.extend({}, this.data);
        },

        get_category_id: function(){
            return this.options.category_id;
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            this.element.find('.category_input').each(function(idx, elem){
                _this.data[$(this).data('key')] = $(this).val();
            });
        },

        _check_data: function(){
            if (this.data['name'].length == 0) {
                alert('Empty field "Category Name"');
                return false;
            }
            return true;
        },

        _mark_dirty: function(){
            if (this.options.category_id == null) {
                // no dirty marking for new category
                return;
            }
            this.element.find('.dirty_marker').addClass('alert-danger');
        },

        _mark_clean: function(){
            this.element.find('.dirty_marker').removeClass('alert-danger');
        },
    });
}(jQuery));
