/* =========================================================
 * edit_login.js
 * =========================================================
 * Copyright 2019 Dario Goetz
 * ========================================================= */

(function($, undefined){
    $.widget('surfjudge.edit_login', {
        options: {
            login_id: null,
            data: null,
            geturl: 'rest/logins/{loginid}',
            posturl: '/rest/logins/{loginid}',
            deleteurl: '/rest/logins/{loginid}',
        },

        _create: function(){
            this.data = this.options.data || {};

            if (!this._check_inputs()){
                console.log('Inputs of edit_login module not valid.');
                return;
            }

            this._init_html();

            this._register_events();
            if (this.options.login_id !== null)
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
            var html = $([
                '<form>',
                '  <div class="form-group row">',
                '    <label class="col-2 col-form-label">Username</label>',
                '    <div class="col-sm-10">',
                '      <input type="text" name="id" class="form-control login_input" data-key="username" placeholder="Username">',
                '    </div>',
                '  </div>',
                '  <div class="form-group row">',
                '    <label class="col-2 col-form-label">Password</label>',
                '    <div class="col-sm-10">',
                '      <input type="password" name="password" class="form-control login_input" data-key="password" placeholder="Password">',
                '    </div>',
                '  </div>',
                '',
                '  <div class="form-group row">',
                '    <label class="col-2 col-form-label">Judge name</label>',
                '    <div class="col-5">',
                '        <input type="text" data-key="first_name" name="first_name" class="form-control login_input" placeholder="First Name">',
                '    </div>',
                '    <div class="col-5">',
                '        <input type="text" data-key="last_name" name="last_name" class="form-control login_input" placeholder="Last Name">',
                '    </div>',
                '  </div>',
                '',
                '  <div class="form-group row">',
                '    <label class="col-2 control-label">Roles</label>',
                '    <div class="col-10 groups_section">',
                '    </div>',
                '  </div>',
                '  <button type="button" class="btn btn-danger delete_btn">Delete</button>',
                '  <div class="float-right">',
                '    <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '    <button type="submit" class="btn btn-primary save_changes_btn">Save changes</button>',
                '  </div>',
                '</div>',
                '</form>',
            ].join(' '));

            var groups = [{label: 'Admin', key: 'ac_admin'}, {label: 'Judge', key: 'ac_judge'}, {label: 'Commentator', key: 'ac_commentator'}];
            $.each(groups, function(idx, role){
                var elem = $('<div>', {class: 'checkbox'})
                    .append($('<label>', {class: 'col-form-label'})
                        .append($('<input>', {class: 'login_input',
                                              type: 'checkbox',
                                              data: {key: role['key']}})
                        ).append($('<span>', {text: role['label']}))
                    );
                html.find('.groups_section').append(elem);
            });

            this.element.append(html);

            if (this.options.login_id == null) {
                this.element.find('.delete_btn').remove();
            }
        },

        _register_events: function(){
            this._on(this.element, {
                'click .reset_btn': this.refresh,
                'click .delete_btn': this.delete_login,
                'click .save_changes_btn': this.upload,
                'change': this._mark_dirty,
                'submit form': function(ev){ev.preventDefault();}, // do not send data, itself
            });
        },

        refresh: function(){
            console.log('refreshing');
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.login_id !== null){
                $.getJSON(this.options.geturl.format({loginid: this.options.login_id}))
                    .done(function(login){
                        if (login != null){
                            _this.data = login;
                            _this._refresh();
                            _this._mark_clean();
                            deferred.resolve();
                        } else {
                            console.log('login not found.');
                            _this.options.login_id = null;
                            _this._mark_clean();
                            deferred.reject();
                        }
                    })
                    .fail(function(){
                        console.log('Connection error.');
                        deferred.reject();
                    });
            } else {
                console.log('Nothing to refresh (no login id specified)');
                _this._refresh();
                _this._mark_clean();
                deferred.resolve();
            }
            return deferred.promise();
        },

        _refresh: function(){
            var _this = this;
            this.data["id"] = this.options.login_id;

            this.element.find('.login_input').each(function(idx, elem){
                var key = $(this).data('key');
                if ($(this).attr('type') == 'checkbox'){
                    if ((_this.data['groups'] || []).indexOf(key) >= 0)
                        $(this).prop('checked', true);
                    else
                        $(this).prop('checked', false);
                }
                else
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
            var post_login_id = this.options.login_id || 'new';
            console.log(this.data);
            $.post(this.options.posturl.format({loginid: post_login_id}), JSON.stringify(this.data), function(login){
                _this.options.login_id = login['id'];
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

        delete_login: function(){
            var _this = this;
            var deferred = $.Deferred();
            if (this.options.login_id !== null) {
                $.ajax({
                    url: this.options.deleteurl.format({loginid: this.options.login_id}),
                    type: 'DELETE',
                })
                .done(function(){
                    // initialize module as empty
                    _this.options.login_id = _this.options.login_id || _this.data['id'];
                    _this.options.login_id = null;
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
                console.log('No login id specified.');
                deferred.reject();
            }
            return deferred.promise();
        },

        _fetch_details_from_inputs: function(){
            var _this = this;
            var groups = [];
            this.element.find('.login_input').each(function(idx, elem){
                var key = $(this).data('key');
                if ($(this).attr('type') == 'checkbox'){
                    if ($(this).prop('checked'))
                        groups.push(key);
                } else {
                    _this.data[key] = $(this).val();
                }
            });
            this.data['groups'] = groups;
        },

        _check_data: function(){
            var pw = this.data['password'];
            if (this.options.login_id == null && (pw.length == 0)) {
               alert('Provide a password for a new login!');
               return false;
            }
            if (this.data['username'].length == 0) {
                alert('Empty field "User"');
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
