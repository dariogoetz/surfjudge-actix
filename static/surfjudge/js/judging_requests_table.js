(function($, undefined){
    $.widget('surfjudge.judging_requests', {
        options: {
            heat_id: null,
            data_requests: null,
            data_assignments: null,

            getjudgingrequestsurl: '/rest/judging_requests',
            getassignedjudgesurl: '/rest/judge_assignments/{heatid}',
            postjudgeassignmentsurl: '/rest/judge_assignments/{heatid}',
            deletejudgeassignmentsurl: '/rest/judge_assignments/{heatid}/{judgeid}',

            websocket_url: null,
        },

        _create: function(){
            this.judging_requests = this.options.data_requests || [];
            this.assigned_judges = this.options.data_assignments || [];
            this.judges = [];

            console.log('Initiating websocket for judging request panel.')
            this.websocket = new WebSocketClient({
                url: this.options.websocket_url,
                channels: {
                    'judging_requests': this.refresh.bind(this),
                },
                name: 'Judging Requests Table',
            });

            this._init_html();
            this._register_events();
            this.refresh();
        },

        _destroy: function(){
            if (this.websocket != null)
                this.websocket.close();
            this.element.empty();
            },

        _init_html: function(){
            var html = [
                '<!-- judging requests -->',
                '<table class="table table-striped judging_requests_table"',
                '       data-toggle="table"',
                '       data-sort-name="judge_id"',
                '       date-sort-order="asc">',
                '    <thead>',
                '        <tr>',
                '            <th data-field="judge_id">Judge ID</th>',
                '            <th data-field="name" data-sortable="true">Judge Name</th>',
                '            <th data-field="expires" data-sortable="true">Expires</th>',
                '            <th data-field="status" data-sortable="true">Status</th>',
                '            <th data-field="action"">Action</th>',
                '        </tr>',
                '    </thead>',
                '</table>',
                '<button class="btn btn-secondary add_judges_btn"><span class="fa fa-plus"></span>&nbsp;Add Judges</button>',
            ].join(' ');

            this.element.append($(html));

            this.element.find('.judging_requests_table').bootstrapTable({
                rowStyle: function rowStyle(row, index){
                    if (row['status'] === 'confirmed')
                        return {classes: 'confirmed'};
                    else if (row['status'] === 'missing')
                        return {classes: 'missing'};
                    else
                        return {classes: 'pending'}
                }
            });
        },

        _register_events: function(){
            this._on(this.element, {
                'click .missing button': function(event){
                    var elem = $(event.currentTarget);
                    this.delete_judge_activity(elem.data('judgeid'));
                },
                'click .confirmed button': function(event){
                    var elem = $(event.currentTarget);
                    this.delete_judge_activity(elem.data('judgeid'));
                },
                'click .pending button': function(event){
                    var elem = $(event.currentTarget);
                    this.confirm_judge_activity(elem.data('judgeid'));
                },
                'click .add_judges_btn': this.add_other_judges,
            });
        },

        refresh: function(){
            var _this = this;
            this.judging_requests = [];
            this.assigned_judges = [];
            var deferred = $.Deferred();

            var deferred_requests = $.Deferred();
            $.getJSON(this.options.getjudgingrequestsurl)
                .done(function(ev_judging_requests){
                    _this.judging_requests = ev_judging_requests;
                    deferred_requests.resolve();
                })
                .fail(function(){
                    console.log('Error getting judging requests.');
                    deferred_requests.resolve();
                });

            var deferred_assignments = $.Deferred();
            $.getJSON(this.options.getassignedjudgesurl.format({heatid: this.options.heat_id}))
                .done(function(ev_assigned_judges){
                    _this.assigned_judges = ev_assigned_judges;
                    deferred_assignments.resolve();
                })
                .fail(function(){
                    console.log('Error getting judge assignments.');
                    deferred_assignments.resolve();
                });
            $.when(deferred_requests, deferred_assignments).done(function(){
                _this._refresh();
                deferred.resolve();
            });
            return deferred.promise();
        },

        _compile_judges: function(){
            var _this = this;

            var res = [];

            var pending = new Map();
            var confirmed = new Map();
            var missing = new Map();

            var requests = new Map();
            // map of judge_ids with active judging requests (confirmed or pending)
            $.each(this.judging_requests, function(idx, val){
                requests.set(val['judge_id'], val);
            });

            // collect confirmed and missing judges (assigmnent with or without request)
            $.each(this.assigned_judges, function(idx, val){
                if (requests.has(val['judge_id'])) {
                    val['expires'] = requests.get(val['judge_id'])['expire_date'].split('.')[0];
                    confirmed.set(val['judge_id'], val);
                } else {
                    missing.set(val['judge_id'], val);
                }
            });

            // collect remaining pending judges (request, but no assigment)
            $.each(this.judging_requests, function(idx, val){
                if ((!confirmed.has(val['judge_id'])) &&
                    (!missing.has(val['judge_id']))) {
                        pending.set(val['judge_id'], val);
                    }
            });

            // compile res with corresponding status
            var push_judges = function(map, status){
                map.forEach(function(val, idx){
                    var d = {};
                    d['judge_id'] = val['judge_id'];
                    d['name'] = val['judge']['first_name'] + ' ' + val['judge']['last_name'];
                    d['expires'] = val['expires'];
                    d['status'] = status;
                    res.push(d);
                });
            };

            push_judges(confirmed, 'confirmed');
            push_judges(missing, 'missing');
            push_judges(pending, 'pending');

            return res;
        },

        _refresh: function(){
            this.judges = this._compile_judges();
            $.each(this.judges, function(idx, val){
                val['action'] = '';
                if (val['status'] === 'pending'){
                    val['action'] = '<button data-judgeid=' + val['judge_id'] + ' class="btn btn-success"><span class="fa fa-check"></span></button>';
                } else if (val['status'] === 'confirmed') {
                    val['action'] = '<button data-judgeid=' + val['judge_id'] + ' class="btn btn-default"><span class="fa fa-times-circle"></span></button>';
                } else if (val['status'] === 'missing') {
                    val['action'] = '<button data-judgeid=' + val['judge_id'] + ' class="btn btn-danger"><span class="fa fa-times-circle"></span></button>';
                }
            });
            this.element.find('.judging_requests_table').bootstrapTable('load', this.judges);
        },

        confirm_judge_activity: function(judge_id){
            var _this = this;

            // using POST appends (or overwrites) to judge assignments for heat
            // using PUT would set this one assignment for the heat (and delete existing ones)
            $.post(this.options.postjudgeassignmentsurl.format({heatid: this.options.heat_id}),
                JSON.stringify([{
                    'judge_id': judge_id,
                    'heat_id': this.options.heat_id,
                }]), function(){
                _this.refresh().done(function(){
                    _this._trigger('data_changed');
                });
            });
        },


        delete_judge_activity: function(judge_id){
            var _this = this;
            $.ajax({
                url: this.options.deletejudgeassignmentsurl.format({heatid: this.options.heat_id, judgeid: judge_id}),
                type: 'DELETE',
            })
                .done(function(){
                    _this.refresh().done(function(){
                        _this._trigger('data_changed');
                    });
                });
        },

        add_other_judges: function(){
            var _this = this;
            var html = [
                '<div class="edit_assigned_judges">',
                '</div>'
            ].join(' ');
            var bb = bootbox.dialog({
                'title': 'Select Judges',
                'message': html,
                'buttons': {
                    cancel: {
                        label: 'Cancel',
                        className: 'btn-default'
                    },
                },
            });

            bb.init(function(){
                var edit_assigned_judges_elem = bb.find('.edit_assigned_judges');
                edit_assigned_judges_elem.edit_assigned_judges({heat_id: _this.options.heat_id});
                edit_assigned_judges_elem.on('edit_assigned_judgesdata_changed', function(){
                    _this.refresh().done(function(){
                        _this._trigger('data_changed');
                        bootbox.hideAll();
                    });
                });
            });
        },
    });
}(jQuery));
