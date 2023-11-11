function doPost(e) {
  try {
    //値の受取り
    const parameter = e.parameter;

    // 現在の日時を取得
    const year = new Date().getFullYear();
    const month = new Date().getMonth() + 1;
    const day = new Date().getDate();
    const hour = new Date().getHours();
    const minute = new Date().getMinutes();
    const sheetName = `${year}-${month}`;
    const datetime = `${year}/${month}/${day} ${hour}:${minute}`;

    const ss = SpreadsheetApp.getActiveSpreadsheet();

    let sheet = ss.getSheetByName(sheetName);
    if (!sheet) sheet = createSheet(sheetName);

    // シートに追記
    sheet.appendRow([datetime, parameter.name, parameter.status]);

    const output = ContentService.createTextOutput();
    output.setMimeType(ContentService.MimeType.JSON);
    output.setContent({ status: "success" });

    return output;
  } catch (error) {
    const output = ContentService.createTextOutput();
    output.setMimeType(ContentService.MimeType.JSON);
    output.setContent({ status: "error", message: error.message });

    return output;
  }
}

// gasでシートを作成
function createSheet(sheetName) {
  const sheet = SpreadsheetApp.getActiveSpreadsheet().insertSheet(sheetName);
  sheet.appendRow(["日付", "名前", "出退勤"]);
  return sheet;
}
