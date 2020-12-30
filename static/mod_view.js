// Mod view script to fill in information like warnings etc.
//console.log(document.getElementById("data_").innerText)
data = JSON.parse(document.getElementById("data_").innerText);
if (data.authors == undefined) data.authors = []
if (data.categories == undefined) data.categories = []
if (data.readme == undefined) data.readme = ""
// set readme
if (data.readme) {
  document.getElementById("content").innerHTML = marked(data.readme);
}

document.getElementById("detail_authors").innerHTML = safetext(data["authors"].join(", "));
document.getElementById("detail_categories").innerHTML = safetext(data["categories"].join(", "));
document.getElementById("verification_level").innerHTML = verificationProperties.fromVerificationLevel(data["verification"]).badge;
document.getElementById("download_button").classList.add(verificationProperties.fromVerificationLevel(data["verification"]).download_colour);

document.getElementById("download_button").setAttribute("onclick", `document.location.href = '${data.files[0]}'`);

var uploaded = new Date(data.uploaded);
document.getElementById("release_date").innerHTML = uploaded.toLocaleDateString()


if (verificationProperties.fromVerificationLevel(data["verification"]).alert) {
  document.getElementById("verification_warnings").innerHTML = verificationProperties.fromVerificationLevel(data["verification"]).alert;
}
