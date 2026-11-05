// Icebreaker : recherche plein texte (index généré dans recherche-index.js) et repli des rubriques.
(function () {
  "use strict";
  var racine = document.body.dataset.racine || "";

  // --- Rubriques repliables, état gardé dans le navigateur ---
  var cle = "icebreaker-wiki-rubriques";
  var repliees = {};
  try { repliees = JSON.parse(localStorage.getItem(cle) || "{}"); } catch (e) {}
  document.querySelectorAll(".nav-rubrique").forEach(function (d) {
    var nom = d.dataset.rubrique;
    if (repliees[nom] && !d.querySelector("li.actif")) d.removeAttribute("open");
    d.addEventListener("toggle", function () {
      repliees[nom] = !d.open;
      try { localStorage.setItem(cle, JSON.stringify(repliees)); } catch (e) {}
    });
  });
  var actif = document.querySelector(".nav-rubrique li.actif");
  if (actif) actif.scrollIntoView({ block: "nearest" });

  // --- Recherche ---
  var champ = document.getElementById("recherche");
  var boite = document.getElementById("recherche-resultats");
  if (!champ || !boite) return;
  var index = window.ICEBREAKER_INDEX || [];

  function normaliser(t) {
    return t.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  }
  index.forEach(function (p) {
    p.titreN = normaliser(p.titre);
    p.texteN = normaliser(p.texte);
  });

  function extrait(texte, texteN, mot) {
    var i = texteN.indexOf(mot);
    if (i < 0) return texte.slice(0, 140);
    var debut = Math.max(0, i - 60);
    var fin = Math.min(texte.length, i + 90);
    return (debut > 0 ? "… " : "") + texte.slice(debut, fin) + (fin < texte.length ? " …" : "");
  }

  function chercher(q) {
    var mots = normaliser(q).split(/\s+/).filter(function (m) { return m.length > 1; });
    if (!mots.length) return [];
    var resultats = [];
    index.forEach(function (p) {
      var score = 0;
      for (var k = 0; k < mots.length; k++) {
        var m = mots[k];
        var dansTitre = p.titreN.indexOf(m) >= 0;
        var dansTexte = p.texteN.indexOf(m) >= 0;
        if (!dansTitre && !dansTexte) return;
        score += dansTitre ? 10 : 0;
        score += dansTexte ? 1 + Math.min(5, p.texteN.split(m).length - 1) : 0;
      }
      resultats.push({ page: p, score: score, mot: mots[0] });
    });
    resultats.sort(function (a, b) { return b.score - a.score; });
    return resultats.slice(0, 12);
  }

  function echapper(t) {
    return t.replace(/[&<>"]/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; });
  }

  function afficher(resultats, q) {
    if (!q) { boite.hidden = true; boite.innerHTML = ""; return; }
    if (!resultats.length) {
      boite.innerHTML = '<div class="recherche-vide">Rien pour « ' + echapper(q) + ' »</div>';
      boite.hidden = false;
      return;
    }
    boite.innerHTML = resultats.map(function (r, i) {
      var p = r.page;
      return '<a class="recherche-item' + (i === 0 ? " choisi" : "") + '" href="' + racine + p.url + '">' +
        '<span class="recherche-rubrique">' + echapper(p.rubrique) + '</span>' +
        '<span class="recherche-titre">' + echapper(p.titre) + '</span>' +
        '<span class="recherche-extrait">' + echapper(extrait(p.texte, p.texteN, r.mot)) + '</span></a>';
    }).join("");
    boite.hidden = false;
  }

  var minuterie = null;
  champ.addEventListener("input", function () {
    clearTimeout(minuterie);
    minuterie = setTimeout(function () { afficher(chercher(champ.value), champ.value.trim()); }, 80);
  });
  champ.addEventListener("focus", function () { if (champ.value.trim()) afficher(chercher(champ.value), champ.value.trim()); });
  champ.addEventListener("keydown", function (e) {
    var items = boite.querySelectorAll(".recherche-item");
    var choisi = boite.querySelector(".recherche-item.choisi");
    var pos = Array.prototype.indexOf.call(items, choisi);
    if (e.key === "Escape") { boite.hidden = true; champ.blur(); }
    else if (e.key === "Enter" && choisi) { window.location.href = choisi.href; }
    else if (e.key === "ArrowDown" && items.length) { e.preventDefault(); if (choisi) choisi.classList.remove("choisi"); items[(pos + 1) % items.length].classList.add("choisi"); }
    else if (e.key === "ArrowUp" && items.length) { e.preventDefault(); if (choisi) choisi.classList.remove("choisi"); items[(pos - 1 + items.length) % items.length].classList.add("choisi"); }
  });
  document.addEventListener("click", function (e) {
    if (!e.target.closest(".recherche")) boite.hidden = true;
  });
  document.addEventListener("keydown", function (e) {
    if (e.key === "/" && document.activeElement !== champ && !/input|textarea/i.test(document.activeElement.tagName)) {
      e.preventDefault();
      champ.focus();
      champ.select();
    }
  });
})();

// Icebreaker : zoom des captures d'écran (W.10). Clic sur l'image : plein écran ; clic ou Échap : retour.
(function () {
  "use strict";
  document.querySelectorAll(".capture a.zoom").forEach(function (lien) {
    lien.addEventListener("click", function (e) {
      e.preventDefault();
      var image = lien.querySelector("img");
      var fond = document.createElement("div");
      fond.className = "zoom-overlay";
      var grande = document.createElement("img");
      grande.src = lien.href;
      grande.alt = image ? image.alt : "";
      var legende = document.createElement("figcaption");
      legende.textContent = grande.alt;
      fond.appendChild(grande);
      fond.appendChild(legende);
      function fermer() {
        fond.remove();
        document.removeEventListener("keydown", surTouche);
      }
      function surTouche(ev) { if (ev.key === "Escape") fermer(); }
      fond.addEventListener("click", fermer);
      document.addEventListener("keydown", surTouche);
      document.body.appendChild(fond);
    });
  });
})();
