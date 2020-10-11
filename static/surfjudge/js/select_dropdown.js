(function($, undefined){
    $.widget('surfjudge.select_dropdown', {
        options: {
            url: null,
            url_requires_options: false,
            label: null,
            data: null,
            select_first_on_load: true,
            update_label: true,
            hide_empty: false,
            remove_selected: false,
            action_callback: null,
            sort_function: function(a,b){
                return (b['start_date'] > a['start_date']) ? 1 : -1;
            },
        },

        _create: function(){
            var _this = this;
            this.selected_value = null;
            this._available_list_items = [];

            this._init_html();
            this._register_events();

            if (this.options.data !== null){
                this.initialized = this.load(this.options.data);
            } else {
                this.initialized = this.refresh();
            }
            this.initialized.done(function(){
            });
        },

        _destroy: function(){
            this.element.empty();
        },

        _init_html: function(){
            var _this = this;
            html = $([
                '<div class="dropdown select_dropdown" style="display:none">',
                '    <div class="btn-group">',
                '    <button class="btn btn-dropdown dropdown-toggle" type="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">',
                '        <span class="dropdown_label"></span>&nbsp;<span class="caret"></span>',
                '    </button>',
                '    <ul class="dropdown-menu scrollable-menu" role="menu">',
                '    </ul>',
                '    </div>',
                '</div>',
            ].join(' '));

            html.find('.dropdown_label').html(this.options.label);
            this.element.append(html);
        },

        load: function(data){
            // refresh using provided data
            var deferred = $.Deferred();
            this.data = data;
            this._available_list_items = this.data.slice();
            this._available_list_items.sort(this.options.sort_function);
            this._refresh();
            if (this.options.select_first_on_load && this._available_list_items.length > 0)
                this.select_item(this.element.find('.dropdown-menu .select').first());
            return deferred.resolve().promise();
        },

        refresh: function(options){
            var _this = this;
            if (this.options.url_requires_options && (!options || $.isEmptyObject(options))){
                this.load([]);
                return $.Deferred().resolve().promise();
            }
            // load data from server and refresh
            var deferred = $.Deferred();
            $.getJSON(this.options.url, options, function(data){
                _this.load(data);
                deferred.resolve();
            });
            return deferred.promise();
        },

        _refresh: function(){
            var l = this.element.find('.dropdown-menu');
            l.empty();
            $.each(this._available_list_items, function(idx, item){
                l.append('<a class="select dropdown-item" href="#" data-index=' + idx + '>' + item['name'] + '</a>');
            });

            this.element.find('.dropdown_label').html(this.options.label);
            if (this._available_list_items.length == 0 && this.options.hide_empty)
                this.element.find('.select_dropdown').hide();
            else
                this.element.find('.select_dropdown').show();
        },

        _register_events: function(){
            this._on(this.element, {
                'click .select': function(ev){
                    this.select_item($(ev.currentTarget));
                },
            })
        },

        select_item: function(elem, id, silent){
            // if id is given, search this._available_list_items for corresponding element
            var selected_item = null;
            if (typeof id !== 'undefined') {
                $.each(this._available_list_items, function(idx, item) {
                    if (item['id'] == id) {
                        selected_item = item;
                    }
                });
            }

            if (selected_item === null && (elem !== null && typeof elem.data('index') !== "undefined")) {
                var idx = elem.data('index');
                selected_item = this._available_list_items[idx];
            }
            if (selected_item !== null) {
                var val = selected_item['id'];
                var label = selected_item['name'];

                if (this.options.remove_selected)
                    this._available_list_items.splice(idx, 1);

                this._refresh();

                if (this.options.update_label)
                    this.element.find('.dropdown_label').html(this.options.label + ' <b>' + label + '</b>');
            } else {
                val = null;
            }
            this.selected_value = val;
            if (!silent) {
                this._trigger('selected', this.selected_value);
                if (this.options.action_callback !== null)
                    this.options.action_callback(this.selected_value);
            }
        },

        get_selected_value: function(){
            return this.selected_value;
        },

        filter: function(filter){
            var filtered_list_items = [];
            $.each(this._available_list_items, function(idx, item){
                if (filter(item))
                    filtered_list_items.push(item);
            });
            this._available_list_items = filtered_list_items;
            this._refresh;
        },

        is_initialized: function(){
            return this.initialized;
        },
    });
}(jQuery));
