(function($, undefined){
    $.widget('surfjudge.csv_upload', {
        options: {
	        required_columns: [],
            expected_columns: [],
            delimiter: "",
        },

        _create: function(){
	        this.import_data = null;
            this.parsed_data = null;
	        this.data_valid = false;

            this._init_html();
            this.reset();
	        this._register_events();
        },

        _destroy: function(){
            this.element.empty();
        },

        _init_html: function(){
            var _this = this;
            var html = $([
		        //'<button type="button" class="btn btn-standard btn-lg btn-file load_btn">Load CSV...<input type="file"></button>',
                '<div>',
                '  <span class="expected_columns"></span>',
                '  <br>',
		        '  <div class="custom-file">',
                '    <input class="custom-file-input input_file" type="file">',
                '    <label class="custom-file-label filename">Choose file</label>',
                '  </div>',
                '  <hr>',
                '  <div class="preview_section">',
                '    <h4>Preview</h4>',
		        '    <div class="csv_preview"></div>',
                '  </div>',
                '  <div class="float-right">',
                '    <button type="button" class="btn btn-light reset_btn">Reset</button>',
                '    <button type="button" class="btn btn-primary submit_btn disabled">Save changes</button>',
                '  </div>',
                '</div>',
            ].join(' '));

            if (this.options.expected_columns.length > 0) {
                html.find('.expected_columns').html('<b>Expecting columns:</b><br>' + this.options.expected_columns.join(', ') + '<br>');
            }

            this.element.append(html);
        },

        reset: function(){
	        this.import_data = null;
            this.parsed_data = null;
	        this.data_valid = false;
            this.element.find('.submit_btn').addClass('disabled');
            this.element.find('.preview_section').hide();
            this.element.find('.filename').text('Choose file...');
            this.element.find('input.input_file').val('');

	        this.show_preview();
	    },

	    confirm: function(){
            if (this.import_data === null)
                return;
	        if (this.data_valid) {
		        this._trigger('data_changed', null, {data: this.parsed_data});
	        } else {
		        bootbox.alert('Data invalid!');
	        }
	    },

	    _register_events: function(){
	        this._on(this.element, {
		        'change input.input_file': this.load_csv,
		        'click .reset_btn': this.reset,
		        'click .submit_btn': this.confirm,
	        });
	    },

	    load_csv: function(ev){
	        var _this = this;
	        var deferred = $.Deferred();
	        let f = ev.target.files[0];
	        Papa.parse(f, {
		        header: true,
                delimiter: this.options.delimiter,
		        complete: function(result){
		            _this.import_data = result;
		            _this.data_valid = false;
                    _this._check_and_import_data();
                    _this.element.find('.filename').text(f.name);
		            _this.show_preview();
		            deferred.resolve();
		        },
	        });
	        return deferred.promise();
	    },

	    _check_and_import_data: function(){
	        var _this = this;

	        // check if all required fields are present
	        var missing_fields = [];
	        var fields = this.import_data.meta.fields;
	        var required_columns_available = true;
	        $.each(this.options.required_columns, function(idx, field){
		        if (fields.indexOf(field) < 0) {
		            required_columns_available = false;
		            missing_fields.push(field);
		        }
	        });

	        this.data_valid = required_columns_available;

	        // import data
	        var res = [];
	        $.each(this.import_data['data'], function(idx, row){
		        var row_okay = true;
		        $.each(_this.options.required_columns, function(idx, field){
		            if (!row[field] || row[field].length == 0) {
			            console.log('Missing field "' + field + '" for row ' + JSON.stringify(row));
			            row_okay = false;
		            }
		        });
		        if (row_okay) {
		            res.push(row);
		        }
	        });
	        if (missing_fields.length > 0) {
                var html = [
                    '<b>Missing columns:</b>',
                    '<br>',
                    '<br>',
                    missing_fields.join('<br> '),
                ].join('');
		        bootbox.alert({
                    title: 'Error reading CSV file',
                    message: html
                });
	        }
            this.parsed_data = res;
	    },

	    show_preview: function(n_preview_lines){
			var _this = this;
			if (n_preview_lines == null) {
				n_preview_lines = 5;
			}
	        this.element.find('.csv_preview').empty();

	        if (this.parsed_data === null){
		        return;
	        } else {
                this.element.find('.submit_btn').removeClass('disabled');
            }

	        var html = $([
		        '<table class="table table-sm table-striped">',
		        '  <thead>',
		        '    <tr>',
		        '    </tr>',
		        '  </thead>',
		        '</table>',
	        ].join(' '));

            var header_row = html.find('tr');
            $.each(this.import_data.meta.fields, function(idx, field){
				if (_this.options.required_columns.indexOf(field) < 0) {
					return;
				}
                header_row.append($('<th data-field="' + field + '">' + field + '</th>'));
            });

	        html.bootstrapTable({data: this.parsed_data.slice(0, n_preview_lines)});
	        this.element.find('.csv_preview').append(html);
            this.element.find('.preview_section').show();
	    },
    });
}(jQuery));
