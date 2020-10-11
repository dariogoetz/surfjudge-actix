(function($, undefined){
    $.widget('surfjudge.publish_button', {
        options: {
            heat_id: null,
            data: null,

            geturl: '/rest/results/{heatid}',
            posturl: '/rest/publish_results/{heatid}',
            deleteurl: '/rest/results/{heatid}',

            class: '',
        },

        _create: function(){
            this._init_html();

            this.data = this.options.data;

            this._register_events();
            this.refresh();
        },

        _destroy: function(){
            this.element.empty();
        },

        _init_html: function(){
            var _this = this;
            html = $([
                '<div class="btn-group">',
                '    <button class="btn btn-secondary btn-lg publish_btn"><span class="fa fa-upload"></span>&nbsp;Publish</button>',
                '    <button class="btn btn-danger btn-lg unpublish_btn"><span class="fa fa-times-circle"></span></button>',
                '</div>',
            ].join(' '));

            html.addClass(this.options.class);

            this.element.append(html);
        },

        refresh: function(){
            var _this = this;
            var deferred = $.Deferred();
            $.getJSON(this.options.geturl.format({heatid: this.options.heat_id}), function(data){
                _this.data = data;
                _this._refresh();
                deferred.resolve();
            });
            return deferred.promise();
        },

        _refresh: function(){
            if (this.data !== null && !$.isEmptyObject(this.data))
                this.element.find('.publish_btn').html('<span class="fa fa-upload"></span>&nbsp;RE-publish');
            else
                this.element.find('.publish_btn').html('<span class="fa fa-upload"></span>&nbsp;Publish');
        },

        _register_events: function(){
            this._on(this.element, {
                'click .publish_btn': this.publish,
                'click .unpublish_btn': this.unpublish,
            })
        },


        publish: function(){
            var _this = this;
            $.post(this.options.posturl.format({heatid: this.options.heat_id}), function(){
                _this.refresh();
            });
        },

        unpublish: function(){
            var _this = this;
            $.ajax({
                url: this.options.deleteurl.format({heatid: this.options.heat_id}),
                type: 'DELETE',
            })
            .done(function(){
                _this.refresh();
            });
        },

    });

}(jQuery));
