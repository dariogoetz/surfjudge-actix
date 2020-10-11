/* =========================================================
* heat_tree_generators.js
* =========================================================
* Copyright 2019 Dario Goetz
* ========================================================= */
class TournamentGenerator {
    constructor(options) {
        this.heatchart_data = null;
        this.options = $.extend({
            number_of_waves: 10,
            duration: 15,
        }, options);
    }

    generate_heatchart_data(n_rounds, participants, relative_seeds) {
        this.heatchart_data = {'heats': [], 'advancements': []};
        console.error('Not implemented!');
    }

    upload(category_id) {
        var _this = this;
        var deferred = $.Deferred();
        var def_surfers = this._upload_surfers();
        $.when(def_surfers).done(function(res_surfer) {
            var def_heats = _this._upload_heats(category_id);
            def_heats.done(function(heat_id_mapping){
                _this._upload_advancements(heat_id_mapping)
                    .done(function(){
                        deferred.resolve();
                    });
            });
        });
        return deferred.promise();
    }

    _fill_seeds(participants, relative_seeds) {
        var _this = this;
        var first_round_heats = this._collect_first_round_heats();
        $.each(participants, function(idx, participant){
            // set global seed:
            // if relative_seeds is set, idx is chosen, participants are filled "tightly"
            // else, seed from data is chosen, some seeds may be left out
            var global_seed;
            if (relative_seeds) {
                global_seed = idx;  // for compact filling
            } else {
                global_seed = parseInt(participant['seed']);  // for allowing holes in filling
            }

            var heat_seed = _this._map_surfer_to_heat_seed(global_seed, first_round_heats.length);

            // add participants map to heats
            var heat = first_round_heats[heat_seed['heat'] % (first_round_heats.length)];
            if (heat['participations'] == null) {
                heat['participations'] = [];
            }

            var p = {};
            p['seed'] = heat_seed['seed'];
            p['surfer'] = participant;
            if (_this.options.preview_lycra_colors != null) {
                var lc = _this.options.preview_lycra_colors;
                var color = lc[p['seed'] % lc.length] || lc[0];
                p['lycra_color'] = color;
            }
            heat['participations'].push(p);
        });
    }

    _upload_surfers() {
        var _this = this;
        var deferred = $.Deferred();
        var deferreds = [];
        $.each(this.heatchart_data['heats'], function(idx, heat){
            if (heat['participations'] != null) {
                $.each(heat['participations'], function(pidx, participant){
                    var def_part = $.Deferred();
                    $.post(_this.options.postsurferurl + '/new', JSON.stringify(participant['surfer']))
                        .done(function(new_part){
                            participant['surfer_id'] = new_part['id'];
                            def_part.resolve();
                        })
                        .fail(function(){
                            console.log('Could not post new surfer');
                            def_part.resolve();
                        });
                    deferreds.push(def_part.promise());
                });
            }
        });
        $.when.apply($, deferreds).done(function(){
            deferred.resolve();
        });
        return deferred.promise();
    }

    _upload_heats(category_id) {
        var _this = this;
        var deferred = $.Deferred();
        var deferreds = [];
        var heat_id_mapping = new Map();
        this.heatchart_data['heats'].sort(function(a, b){
            if ([a['round'], a['number_in_round']] < [b['round'], b['number_in_round']]) {
                return -1;
            } else {
                return 1;
            }
        });
        $.each(this.heatchart_data['heats'], function(idx, heat){
            var upload_data = {};
            upload_data['name'] = heat['name'];
            upload_data['heat_type'] = heat['heat_type'] || 'standard';
            upload_data['round'] = heat['round'];
            upload_data['number_in_round'] = heat['number_in_round'];
            upload_data['category_id'] = category_id;
            upload_data['duration'] = _this.options["duration"];
            upload_data['number_of_waves'] = _this.options["number_of_waves"];
            var deferred_heat = $.Deferred();
            $.post(_this.options.postheaturl + '/new', JSON.stringify(upload_data), function(res_heat){
                heat_id_mapping.set(heat['id'], res_heat['id']);
                if (heat['participations'] != null) {
                    _this._upload_participants_for_heat(heat['participations'], res_heat['id'])
                        .done(function(){
                            deferred_heat.resolve();
                        })
                        .fail(function(){
                            deferred_heat.resolve();
                        });
                } else {
                    // no participants need to be uploaded
                    deferred_heat.resolve();
                }
            });
            deferreds.push(deferred_heat.promise());
        });
        $.when.apply($, deferreds).done(function(){
            deferred.resolve(heat_id_mapping);
        });
        return deferred.promise();
    }

    _upload_participants_for_heat(participants, heat_id) {
        var _this = this;
        var deferred = $.Deferred();
        var participant_data = [];
        $.each(participants, function(idx, participant){
            var p = {};
            p['seed'] = participant['seed'];
            p['surfer_id'] = participant['surfer_id'];
            p['heat_id'] = heat_id;
            // let server chose a color
            participant_data.push(p);
        });
        $.ajax({
            type: 'PUT',
            url: _this.options.putparticipantsurl.format({heatid: heat_id}),
            data: JSON.stringify(participant_data),
        })
        .done(function(){deferred.resolve();})
        .fail(function(){
            console.log('Could not PUT participants for heat', heat_id);
            deferred.resolve();
        });
        return deferred.promise();
    }

    _upload_advancements(heat_id_mapping) {
        var rules = [];
        var deferred = $.Deferred();
        $.each(this.heatchart_data['advancements'], function(idx, advancement){
            var data = {};
            data['seed'] = advancement['seed'];
            data['place'] = advancement['place'];
            data['to_heat_id'] = heat_id_mapping.get(advancement['to_heat_id']);
            data['from_heat_id'] = heat_id_mapping.get(advancement['from_heat_id']);
            rules.push(data);
        });
        $.post(this.options.postadvancementsurl, JSON.stringify(rules), function(res){
            deferred.resolve();
        });
        return deferred;
    }

    _map_surfer_to_heat_seed(seed_idx, nheats_first_lvl) {
        var res_heat = 0;
        for (var level = 0; level < Math.ceil(Math.log2(nheats_first_lvl)); level++)
            res_heat += Math.floor(seed_idx % nheats_first_lvl / 2**(level) % 2) * Math.ceil(nheats_first_lvl/ 2**(level+1));
        var res_seed = Math.floor(seed_idx / nheats_first_lvl);
        return {heat: res_heat, seed: res_seed}
    }

    _collect_first_round_heats() {
        var max_round = 0;
        $.each(this.heatchart_data['heats'], function(idx, heat){
            max_round = Math.max(max_round, heat['round']);
        });
        var res = [];
        $.each(this.heatchart_data['heats'], function(idx, heat){
            if (heat['round'] == max_round) {
                res.push(heat);
            }
        });
        return res;
    }
}

class CustomTournamentGenerator extends TournamentGenerator {
    constructor(options) {
        super(options);
        this._total_rounds = null;
    }

    generate_heatchart_data(round_options, participants, relative_seeds) {
        var _this = this;
        this.heatchart_data = {advcancements: [], heats: []}
        var id = 0;
        $.each(round_options, function(idx, options){
            for (var number_in_round = 0; number_in_round < options['heats_in_round']; number_in_round++) {
                var heat = {}
                heat['id'] = id++;
                heat['number_in_round'] = number_in_round;
                heat['round'] = options['round'];
                heat['name'] = options['name'].format({number: number_in_round + 1});

                _this.heatchart_data['heats'].push(heat);
            }
        });
        if (participants != null) {
            this._fill_seeds(participants, relative_seeds);
        }
        return this.heatchart_data;
    }

    _fill_seeds(participants, relative_seeds) {
        var _this = this;
        var first_round_heats = this._collect_first_round_heats();
        $.each(participants, function(idx, participant){
            // set global seed:
            // if relative_seeds is set, idx is chosen, participants are filled "tightly"
            // else, seed from data is chosen, some seeds may be left out
            var global_seed;
            if (relative_seeds) {
                global_seed = idx;  // for compact filling
            } else {
                global_seed = parseInt(participant['seed']);  // for allowing holes in filling
            }

            var heat_seed = {seed: parseInt(global_seed / first_round_heats.length), heat: global_seed % first_round_heats.length};

            // add participants map to heats
            var heat = first_round_heats[heat_seed['heat'] % (first_round_heats.length)];
            if (heat['participations'] == null) {
                heat['participations'] = [];
            }

            var p = {};
            p['seed'] = heat_seed['seed'];
            p['surfer'] = participant;
            if (_this.options.preview_lycra_colors != null) {
                var lc = _this.options.preview_lycra_colors;
                var color = lc[p['seed'] % lc.length] || lc[0];
                p['lycra_color'] = color;
            }
            heat['participations'].push(p);
        });
    }
}

class StandardTournamentGenerator extends TournamentGenerator {
    constructor(options) {
        super(options);
        this.heat_structure_data = null;
        this._total_rounds = null;
    }

    generate_heatchart_data(n_rounds, participants, relative_seeds) {
        this._generate_heat_structure(n_rounds);
        // generate arrays for advancement data and heat data as received from server
        if (this.heat_structure_data === null)
            return;
        var _this = this;

        var heats = [];
        this.heat_structure_data['heats'].forEach(function(heat, key){
            var new_heat = $.extend({}, heat);
            heats.push(new_heat)
        });
        var advancements = [];
        this.heat_structure_data['advancing_surfers'].forEach(function(heats, level){
            $.each(heats, function(idx, heat_advancements){
                var target_heat = _this.heat_structure_data['heats'].get(String([level, idx]));
                var advancement_rules = {};
                $.each(heat_advancements, function(seed, origin){
                    var origin_heat = _this.heat_structure_data['heats'].get(String([origin['round'], origin['heat']]));

                    // prepare advancement rule for this seed
                    advancements.push({
                        to_heat_id: target_heat['id'],
                        from_heat_id: origin_heat['id'],
                        seed: parseInt(seed),
                        place: origin['place'],
                        name: 'Seed ' + (seed+1),
                    });
                });
            });
        });
        this.heatchart_data = {'advancements': advancements, 'heats': heats}
        if (participants != null) {
            this._fill_seeds(participants, relative_seeds);
        }
        return this.heatchart_data;
    }

    _generate_heat_structure(n_rounds) {
        // generates an object with fields
        // "advancing_surfers": map round -> {heat_idx: advancement_data and
        // "heats": map heat_idx -> {round, heat, id, name}
        var _this = this;
        this._total_rounds = n_rounds;
        var heat_idx = 0;
        // heats is a hash map round+heat to (temporary) index and name
        var heats = new Map();
        heats.set(String([0, 0]), {
            round: 0,
            heat: 0,
            number_in_round: 0,
            id: heat_idx++,
            name: _this._gen_heat_name(0, 0, 0)})

        var advancements = new Map();
        var rnd = 0;
        var n_heats_for_rnd = 1;
        while (rnd < n_rounds - 1) {
            for (var hidx = 0; hidx < n_heats_for_rnd; hidx++) {
                var advancing_from = {};
                for (var seed = 0; seed < 4; seed++) {
                    var adv = {};
                    adv[seed] = this._advances_from(rnd, hidx, seed);
                    advancing_from = $.extend(true, advancing_from, adv);
                    var from_rnd = advancing_from[seed]['round'];
                    var from_heat = advancing_from[seed]['heat'];

                    if (!heats.has(String([from_rnd, from_heat]))) {
                        heats.set(String([from_rnd, from_heat]), {
                            round: from_rnd,
                            heat: from_heat,
                            number_in_round: from_heat,
                            id: heat_idx++,
                            name: _this._gen_heat_name(from_rnd, from_heat, n_rounds),
                        });
                    }
                }
                if (!(advancements.has(rnd))){
                    advancements.set(rnd, {});
                }
                advancements.get(rnd)[hidx] = advancing_from;
            }
            rnd += 1;
            n_heats_for_rnd = 2**rnd;
        }
        this.heat_structure_data = {advancing_surfers: advancements, heats: heats};
        return this.heat_structure_data;
    }

    _advances_from(rnd, hidx, seed) {
        var from_rnd = rnd+1;
        var from_place = Math.floor(seed/2);
        var from_heat;
        if (seed < 2 || rnd == 0)
            from_heat = 2*hidx + seed%2;
        else {
            if (seed < 4)
                from_heat = 2 * (hidx - (hidx%2) + (hidx+1)%2) + seed%2;
            else
                console.log('Error: More than 4 seeds');
        }
        return {round: from_rnd, heat: from_heat, place: from_place};
    }

    _gen_heat_name(rnd, hidx, n_rounds) {
        var res = '';
        if (rnd==0) {
            res = 'Final';
        } else if (rnd == 1) {
            res = 'Semi-Final ' + (hidx+1);
        } else if (rnd == 2) {
            res = 'Quarter-Final ' + (hidx+1);
        } else {
            res = 'Round ' + (n_rounds - rnd) + ' Heat ' + (hidx + 1);
        }
        return res;
    }
}


class RSLTournamentGenerator extends TournamentGenerator {
    constructor(options) {
        super(options);
        this._total_rounds = null;
        this._heat_idx = null;
        this._total_rounds = null;
        this._heat_idx = null;
    }

    generate_heatchart_data(n_rounds, participants, relative_seeds) {
        this.heatchart_data = {heats: [], advancements: []};
        this._total_rounds = n_rounds;
        this._heat_idx = 0;
        this._generate_call_rec(0, n_rounds, 0, 1);
        if (participants != null) {
            this._fill_seeds(participants, relative_seeds);
        }
        return this.heatchart_data;
    }

    _round_name(round) {
        if (round == this._total_rounds) {
            return 'Final';
        } else if (round == this._total_rounds - 1) {
            return 'Semi-Final';
        } else if (round == this._total_rounds - 2) {
            return 'Quarter-Final'
        } else {
            return 'Round {0}'.format(round);
        }
    }

    _generate_call_rec(branch, n_remaining_rounds, place_first_link, place_second_link) {
        var place_first_link = place_first_link == null ? 1 : place_first_link;
        var place_second_link = place_second_link == null ? 2 : place_second_link;

        var last_cut = this._generate_cut_rec(branch, n_remaining_rounds);
        var heat_id = this._heat_idx++;
        var heat = {
            id: heat_id,
            heat_type: 'call',
            round: 2 * (this._total_rounds - n_remaining_rounds),
            number_in_round: branch,
            name: '{0} Call {1}'.format(this._round_name(n_remaining_rounds), branch + 1),
        };
        this.heatchart_data['heats'].push(heat);
        var link_first = {
            from_heat_id: last_cut['id'],
            to_heat_id: heat_id,
            place: place_first_link,
            seed: 0,
        };
        var link_second = {
            from_heat_id: last_cut['id'],
            to_heat_id: heat_id,
            place: place_second_link,
            seed: 1,
        };
        this.heatchart_data['advancements'].push(link_first);
        this.heatchart_data['advancements'].push(link_second);

        return {'call': heat, 'cut': last_cut};
    }

    _generate_cut_rec(branch, n_remaining_rounds) {
        var heat_id = this._heat_idx++;
        var heat = {
            id: heat_id,
            heat_type: 'standard',
            round: 2 * (this._total_rounds - n_remaining_rounds) + 1,
            number_in_round: branch,
            name: '{0} Cut {1}'.format(this._round_name(n_remaining_rounds), branch + 1),
        };
        this.heatchart_data['heats'].push(heat);

        if (n_remaining_rounds > 1) {
            for (var seed = 0; seed < 2; seed++) {
                var data = this._generate_call_rec(2*branch + seed, n_remaining_rounds - 1);
                var last_call = data['call'];
                var last_cut = data['cut'];

                var link_direct = {
                    from_heat_id: last_cut['id'],
                    to_heat_id: heat_id,
                    place: 0,
                    seed: seed,
                };
                this.heatchart_data['advancements'].push(link_direct);

                var link_call = {
                    from_heat_id: last_call['id'],
                    to_heat_id: heat_id,
                    place: 0,
                    seed: seed + 2,
                };
                this.heatchart_data['advancements'].push(link_call);
            }
        }
        return heat;
    }
}
