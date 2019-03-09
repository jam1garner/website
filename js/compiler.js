colors = [ "#a52a2a", "#00ffff", "#008b8b", "#a9a9a9", "#006400", "#bdb76b", "#8b008b", "#556b2f", "#ff8c00", "#9932cc", "#8b0000", "#e9967a", "#9400d3", "#ff00ff", "#ffd700", "#008000", "#4b0082", "#f0e68c", "#add8e6", "#e0ffff", "#90ee90", "#d3d3d3", "#ffb6c1", "#ffffe0", "#00ff00", "#ff00ff", "#800000", "#000080", "#808000", "#ffa500", "#ffc0cb", "#800080", "#800080", "#ff0000", "#c0c0c0", "#ffffff", "#ffff00" ];

usedColors = []

function darken(col, amt) {
    console.log("in - "+col);
    amt = -amt;
    var usePound = false;
    if (col[0] == "#") {
        col = col.slice(1);
        usePound = true;
    }
    var num = parseInt(col,16);
    var r = (num >> 16) + amt;
    if (r > 255) r = 255;
    else if  (r < 0) r = 0;
    var b = ((num >> 8) & 0x00FF) + amt;
    if (b > 255) b = 255;
    else if  (b < 0) b = 0;
    var g = (num & 0x0000FF) + amt;
    if (g > 255) g = 255;
    else if (g < 0) g = 0;
    var outStr = (g | (b << 8) | (r << 16)).toString(16);
    var outStr = (usePound?"#":"") + "0".repeat(6 - outStr.length) + (g | (b << 8) | (r << 16)).toString(16);
    console.log("out - "+outStr);
    return outStr;
}

function enableTabbing(){
    var textarea = document.getElementById('c-code');
    textarea.onkeydown = tabKeyDown;
}

function tabKeyDown(e) {
    if (e.keyCode === 9) { // tab key
        e.preventDefault();  // this will prevent us from tabbing out of the editor

        // now insert four non-breaking spaces for the tab key
        var editor = document.getElementById("c-code");
        var doc = editor.ownerDocument.defaultView;
        var sel = doc.getSelection();
        var range = sel.getRangeAt(0);

        var tabNode = document.createTextNode("\u00a0\u00a0\u00a0\u00a0");
        range.insertNode(tabNode);

        range.setStartAfter(tabNode);
        range.setEndAfter(tabNode); 
        sel.removeAllRanges();
        sel.addRange(range);
    }
}

function get_colors(msc) {
    var lineNums = [];
    msc.scripts.forEach((script) => {
        script.commands.forEach((command) => {
            lineNum = parseInt(command.split(" ")[0]);
            if (lineNums.indexOf(lineNum) == -1) {
                lineNums.push(lineNum);
            }
        });
    });
    
    lineNums.sort();

    var i = 0;
    var color_mapping = {};
    lineNums.forEach((lineNum) => {
        color_mapping[lineNum] = darken(colors[i], 100);
        i += 1;
        i = i % colors.length;
    });
    
    return color_mapping;
}

function add(tbody, text, color) {
    var row = tbody.insertRow();
    var cell = row.insertCell();
    cell.colspan = 2;
    cell.innerText = text;
    if(color != "none")
        row.style["background"] = color;
}

function add_colored(tbody, text1, text2, bgcolor) {
    var row = tbody.insertRow();
    var cell1 = row.insertCell();
    var cell2 = row.insertCell();
    var highlight1 = document.createElement("span");
    var highlight2 = document.createElement("span");
    highlight1.classList.add("highlight1");
    highlight1.innerText = text1;
    highlight2.classList.add("highlight2");
    if(text2 == undefined)
        text2 = "";
    highlight2.innerText = text2;
    cell1.appendChild(highlight1);
    cell2.appendChild(highlight2);
    if(bgcolor != "none")
        row.style["background"] = bgcolor;
}

function display_result(msc) {
    if(msc.error_type != 0) {
        console.log(msc);
        old_tbody = document.getElementsByTagName('tbody')[0];
        var new_tbody = document.createElement('tbody');
        add(new_tbody, msc.error, "none");
        old_tbody.parentNode.replaceChild(new_tbody, old_tbody); 
        return;
    }
    old_tbody = document.getElementsByTagName('tbody')[0];
    var new_tbody = document.createElement('tbody');
    
    var line_colors = get_colors(msc);
    console.log(line_colors);

    msc.scripts.forEach((script) => {
        add(new_tbody, script.name + ":", "none");
        script.commands.forEach((command) => {
            var split_command = command.split(" ");
            var com_color = line_colors[parseInt(split_command[0])];
            add_colored(new_tbody, split_command[1], split_command[2], com_color);
        });
        add(new_tbody, "\xa0", "none");
    });
    msc.strings.forEach((string) => {
        add_colored(new_tbody, ".string ", '"'+string+'"', "none");
    });

    // Apply colors to orignal code
    var codeTextArea = document.getElementById('c-code');
    codeTextArea.childNodes.forEach((node) => {
        node.style["background"] = "none";
    });
    
    for(lineNum in line_colors) {
        codeTextArea.childNodes[lineNum - 1].style["background"] = line_colors[lineNum];
    }

    old_tbody.parentNode.replaceChild(new_tbody, old_tbody); 
}

function compile() {
    var codeTextArea = document.getElementById('c-code');
    var code = codeTextArea.innerHTML
                    .replace(/<div(?: [^>]*?)?><br><\/div>/g, "\n")
                    .replace(/<div(?: [^>]*?)?>((?:.|\n)*?)<\/div>/g, "$1\n")
                    .replace(/&nbsp;/g, " ")
                    .replace(/\xa0/g, " ")
                    .replace(/<br>/g, "");
    console.log(code);
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
    compile();
}
