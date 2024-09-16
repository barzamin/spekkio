
const elDropTarget = document.getElementById('drop-region');
elDropTarget.addEventListener('dragover', function (evt) {
    evt.preventDefault();
});
elDropTarget.addEventListener('drop', function (evt) {
    evt.preventDefault();
    elDropTarget.classList.remove('dragover');

    const item = evt.dataTransfer.items[0];
    if (item.kind != 'file') return;
    const file = item.getAsFile();
    console.log(file);
});

// the drag&drop api is lowkey dogshit
let enteredTarget;
elDropTarget.addEventListener('dragenter', function (evt) {
    enteredTarget = evt.target;
    elDropTarget.classList.add('dragover');
});
elDropTarget.addEventListener('dragleave', function (evt) {
    if (evt.target == enteredTarget) {
        elDropTarget.classList.remove('dragover');
    }
})