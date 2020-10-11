(function($, undefined){
    // allow string formatting, e.g. "hallo {0} and {1}".format('you', 'them') -> "hallo you and them"
    // or "hallo {a} and {b}".format({a: 'you', b: 'them'}) -> "hallo you and them"
    String.prototype.format = function() {
        var args = arguments;
        return this.replace(/\{([^}]+?)\}/g, function(_, match) {
            if (typeof args[0] == 'object') {
                return args[0][match];
            } else {
                return args[match];
            }
        });
    };
}(jQuery));
