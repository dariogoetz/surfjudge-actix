/* =========================================================
 * edit_lycra_color.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_lycra_color', {
        options: {
            lycra_color_id: null,
            data: null,
            geturl: 'rest/lycra_colors/{lycraid}',
            posturl: '/rest/lycra_colors/{lycraid}',
            deleteurl: '/rest/lycra_colors/{lycraid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_lycra_color module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.lycra_color_id !== null)
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
                '        <label class="col-3 control-label">Color Name</label>',
                '        <div class="col-9">',
                '            <input type="hidden" class="lycra_color_input" data-key="id">',
                '            <input type="text" name="name" class="form-control lycra_color_input" data-key="name" placeholder="Color Name">',
                '        </div>',
                '    </div>',
                '    <div class="form-group row">',
                '        <label class="col-3 control-label">Seed</label>',
                '        <div class="col-9">',
                '            <input type="integer" name="seed" class="form-control lycra_color_input" data-key="seed" placeholder="Seed">',
                '        </div>',
                '    </div>',
                '    <div class="form-group row">',
                '        <label class="col-3 control-label">HEX Color</label>',
                '        <div class="col-6">',
                '            <input type="text" name="hex" class="form-control lycra_color_input" data-key="hex" placeholder="HEX Color Definition">',
                '        </div>',
                '        <div class="col-3 color_preview">&nbsp;</div>',
                '    </div>',
                '    <br>',
                '    <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '    <div class="float-right">',
                '        <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '        <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '    </div>',
                '</div>',
                '</form>',
            ].join(' ');

            this.element.append(html);

            if (this.options.lycra_color_id == null) {
                this.element.find('.delete_btn').remove();
            }
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_lycra_color,
                'click .save_changes_btn': this.upload,
                'change .lycra_color_input[data-key="hex"]': this._update_color_preview,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.lycra_color_id !== null){
                $.getJSON(this.options.geturl.format({lycraid: this.options.lycra_color_id}))
                    .done(function(lycra_color){
                        if (lycra_color != null){
                            _this.data = lycra_color;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('Lycra color not found.');
                            _this.options.lycra_color_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no lycra color id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.element.find('.lycra_color_input').each(function(idx, elem){
                var key = $(this).data('key');
                $(this).val(_this.data[key]);
            });
            this._update_color_preview();
        },

        _update_color_preview: function(ev){
            var hex = this.element.find('.lycra_color_input[data-key="hex"]').val();
            this.element.find('.color_preview').attr('style', 'background-color: {0}'.format(hex));
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
            var post_lycra_color_id = this.options.lycra_color_id || 'new';
            $.post(this.options.posturl.format({lycraid: post_lycra_color_id}), JSON.stringify(this.data), function(lycra_color){
                _this.options.lycra_color_id = lycra_color['id'];
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

        delete_lycra_color: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.lycra_color_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({lycraid: this.options.lycra_color_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    // initialize module as empty
                    _this.options.lycra_color_id = _this.options.lycra_color_id || _this.data['id'];
                    _this.options.lycra_color_id = null;
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
                console.log('No lycra color id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            this.element.find('.lycra_color_input').each(function(idx, elem){
                _this.data[$(this).data('key')] = $(this).val();
            });
        },

        _check_data: function(){
            if (this.data['name'].length == 0) {
                alert('Empty field "Color Name"');
                return false;
            }
            return true;
        },

        _mark_dirty: function(){
            if (this.options.lycra_color_id == null) {
                // no dirty marking for new lycra color
                return;
            }
            this.element.find('.dirty_marker').addClass('alert-danger');
        },

        _mark_clean: function(){
            this.element.find('.dirty_marker').removeClass('alert-danger');
        },
    });
}(jQuery));
