const fs = require('fs');
let code = fs.readFileSync('src/App.vue', 'utf8');
code = code.replace(/appWindow\.startDragging\(direction as any\);/g, 'appWindow.startDragging();');
fs.writeFileSync('src/App.vue', code);
