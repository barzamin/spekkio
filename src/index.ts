import './style.css';

import * as rust from '../rust/pkg/index.js';

rust.init();
const audio_decoder = rust.AudioDecoder.new();

const elDropTarget = document.getElementById('drop-region') as HTMLDivElement;
const elSpekkio = document.getElementById('spekkio') as HTMLImageElement;

// don't let them pick him up!!
elSpekkio.addEventListener('dragstart', (evt) => evt.preventDefault());

elDropTarget.addEventListener('dragover', function (evt) {
    console.log('dragover');
    evt.preventDefault();
});
elDropTarget.addEventListener('drop', function (evt) {
    evt.preventDefault();
    elDropTarget.classList.remove('dragover');

    const item = evt.dataTransfer.items[0];
    if (item.kind != 'file') return;
    const file = item.getAsFile();

    file.arrayBuffer().then(buf => {
        audio_decoder.decode(new Uint8Array(buf), item.type);
    });
});

// the drag&drop api is lowkey dogshit
let enteredTarget: EventTarget = undefined;
elDropTarget.addEventListener('dragenter', function (evt) {
    enteredTarget = evt.target;
    elDropTarget.classList.add('dragover');
});
elDropTarget.addEventListener('dragleave', function (evt) {
    if (evt.target == enteredTarget) {
        elDropTarget.classList.remove('dragover');
    }
});
