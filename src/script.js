(function () {
    "use strict";

    const input_url = document.querySelector("#url");
    const a_svg = document.querySelector("#svg");
    const a_pdf = document.querySelector("#pdf");
    const a_png = document.querySelector("#png");
    const img_svg = a_svg.querySelector("img");
    const img_pdf = a_pdf.querySelector("img");
    const img_png = a_png.querySelector("img");

    const NO_TIMEOUT = {};
    let timeout = NO_TIMEOUT;
    let old_url = input_url.value;

    function prevent_default (ev) {
        ev.preventDefault();
        return false;
    }

    function onchange () {
        if (timeout !== NO_TIMEOUT) {
            clearTimeout(timeout);
            timeout = NO_TIMEOUT;
        }
        timeout = setTimeout(onchange_do, 500);
    };

    function onchange_do () {
        timeout = NO_TIMEOUT;
        let new_url = input_url.value;
        if (new_url !== old_url) {
            old_url = null;
            let q = encodeURIComponent(new_url);
            img_svg.src = img_pdf.src = img_png.src = a_svg.href = `./kurzlink.svg?q=${q}`;
            a_pdf.href = `./kurzlink.pdf?q=${q}`;
            a_png.href = `./kurzlink.png?q=${q}`;
            old_url = new_url;
        }
    }

    document.querySelector("form").addEventListener("submit", prevent_default, true);
    for (let event of ["blur", "change", "cut", "input", "keydown", "keypress", "keyup", "paste"]) {
        input_url.addEventListener(event, onchange);
    }
} ());
