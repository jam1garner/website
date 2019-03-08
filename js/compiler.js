function enableTabbing(){
    var textareas = document.getElementsByTagName('textarea');
    var count = textareas.length;
    for(var i=0;i<count;i++){
        textareas[i].onkeydown = function(e){
            if(e.keyCode==9 || e.which==9){
                e.preventDefault();
                var s = this.selectionStart;
                this.value = this.value.substring(0,this.selectionStart) + "    " + this.value.substring(this.selectionEnd);
                this.selectionEnd = s+4; 
            }
        }
    }
}

function display_result(msc) {
    console.log(msc);
}

function compile() {
    var codeTextArea = document.getElementById('c-code');
    var code = codeTextArea.value;
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            display_result(JSON.parse(this.responseText));
        }
    };
    xhttp.open("POST", "/compile", true);
    xhttp.send(code); 
}

window.onload = function() {
    enableTabbing();

    document.getElementById('compile').onclick = compile;
}
