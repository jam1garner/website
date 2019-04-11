function dragenter(e) {
    e.stopPropagation();
    e.preventDefault();
    dropbox = document.getElementById("upload");
    dropbox.style["border"] = "solid 1px blue";
}

function dragover(e) {
    e.stopPropagation();
    e.preventDefault();
}

function dragleave(e) {
    dropbox = document.getElementById("upload");
    dropbox.style["border"] = "solid 1px red";
}

function drop(e) {
    e.stopPropagation();
    e.preventDefault();
    dropbox = document.getElementById("upload");
    dropbox.style["border"] = "solid 1px red";

    const dt = e.dataTransfer;
    const files = dt.files;

    handleFiles(files);
} 

function FileUpload(file) {
    const xhr = new XMLHttpRequest();
    this.xhr = xhr;

    const self = this;
    this.xhr.upload.addEventListener("progress", function(e) {
        if (e.lengthComputable) {
            const percentage = Math.round((e.loaded * 100) / e.total);
            document.getElementById("progress").innerText = "Progress: " + percentage + "%";
        }
    }, false);

    xhr.onreadystatechange = function() {
        if (this.status == 404) {
            document.getElementById("progress").innerText = "Failed, make sure your file is 64 kbps and a valid upload.";
        }
        if (this.readyState == 4 && this.status == 200) {
            window.open(this.response);
            document.getElementById("progress").innerText = "Success";
        }
    };
    xhr.open("POST", "/nus3audio/upload?name=" + file.name);
    xhr.overrideMimeType('text/plain; charset=x-user-defined-binary');
    xhr.send(file);
}

function handleFiles(files) {
    const file = files[0];
    console.log(file);
    new FileUpload(file);
}

function enableDragDrop() {
    let dropbox;

    dropbox = document.getElementById("upload");
    dropbox.addEventListener("dragenter", dragenter, false);
    dropbox.addEventListener("dragexit", dragleave, false);
    dropbox.addEventListener("dragleave", dragleave, false);
    dropbox.addEventListener("dragover", dragover, false);
    dropbox.addEventListener("drop", drop, false);
}

window.onload = function() {
    enableDragDrop();
    document.getElementById('compile').onclick = compile;
    compile();
}
