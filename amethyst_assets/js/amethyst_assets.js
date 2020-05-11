function asset_fetch(path) {
    var xhr = new XMLHttpRequest();
    xhr.responseType = 'arraybuffer';

    xhr.open('GET', path, false);
    xhr.send();

    if (xhr.status === 200) {
        return new Uint8Array(xhr.response);
    } else {
        console.error("asset fetch failed: " + xhr.status + " " + xhr.statusText);
        return new Uint8Array();
    }
}

if (typeof exports === 'object' && typeof module === 'object')
    module.exports = asset_fetch;
else if (typeof define === 'function' && define['amd'])
    define([], function() { return asset_fetch; });
else if (typeof exports === 'object')
    exports["asset_fetch"] = asset_fetch;
