function web_socket_send(web_socket, src) {
    // Turn the array view into owned memory.
    var standalone = [...src];
    // Make it a Uint8Array.
    let bytes = new Uint8Array(standalone);

    console.log("Bytes to send: "+ bytes);
    web_socket.send(bytes);
}

if (typeof exports === 'object' && typeof module === 'object')
    module.exports = bytes_owned;
else if (typeof define === 'function' && define['amd'])
    define([], function() { return bytes_owned; });
else if (typeof exports === 'object')
    exports["bytes_owned"] = bytes_owned;
