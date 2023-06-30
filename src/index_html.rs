markup::define! {
    IndexHtml {
        @markup::doctype()
        html[lang="de-DE"] {
            head {
                meta[charset="UTF-8"];
                title { "QR-Code-Generierer" }

                meta[name="viewport", content="width=device-width, initial-scale=1"];
                meta["http-equiv"="X-UA-Compatible", content="IE=edge"];
                meta["http-equiv"="expires", content="Sat, 01 Dec 2001 00:00:00 GMT"];
                meta["http-equiv"="cache-control", content="no-cache, no-store, must-revalidate" ];
                meta["http-equiv"="pragma", content="no-cache"];

                link[rel="stylesheet", href="./modern-normalize.css"];
                link[rel="stylesheet", href="./style.css"];
            }
            body {
                h1 {
                    "QR-Code-Generierer"
                }
                form[id="form"] {
                    fieldset {
                        div {
                            label[for="url"] {
                                "URL:"
                            }
                            " "
                            label {
                                input[
                                    type="url",
                                    value="https://www.vetmed.fu-berlin.de/",
                                    autocomplete="off",
                                    spellcheck="off",
                                    id="url",
                                ];
                            }
                        }
                    }
                }
                div[style="text-align: center"] {
                    div.grid {
                        p[style="margin-left: auto;"] {
                            "SVG:"
                            br;
                            a[
                                href="./kurzlink.svg?q=https://www.vetmed.fu-berlin.de/",
                                download="kurzlink.svg",
                                title="Klicken um SVG zu speichern.",
                                id="svg",
                            ] {
                                img[
                                    src="./kurzlink.svg?q=https://www.vetmed.fu-berlin.de/",
                                    alt="SVG",
                                ];
                            }
                        }
                        p {
                            "PNG:"
                            br;
                            a[
                                href="./kurzlink.png?q=https://www.vetmed.fu-berlin.de/",
                                download="kurzlink.png",
                                title="Klicken um PNG zu speichern.",
                                id="png",
                            ] {
                                img[
                                    src="./kurzlink.svg?q=https://www.vetmed.fu-berlin.de/",
                                    alt="PNG",
                                ];
                            }
                        }
                        p[style="margin-right: auto;"] {
                            "PDF:"
                            br;
                            a[
                                href="./kurzlink.pdf?q=https://www.vetmed.fu-berlin.de/",
                                download="kurzlink.pdf",
                                title="Klicken um PDF zu speichern.",
                                id="pdf",
                            ] {
                                img[
                                    src="./kurzlink.svg?q=https://www.vetmed.fu-berlin.de/",
                                    alt="PDF",
                                ];
                            }
                        }
                    }
                    p.hint {
                        "Zum Speichern auf das Bild klicken und einen Moment warten. \
                         Wenn möglich, sollten Sie PDFs oder SVGs bevorzugen."
                    }
                }
                script[async, defer, src="./script.js"] {}
            }
        }
    }
}
