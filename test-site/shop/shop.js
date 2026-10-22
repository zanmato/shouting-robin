// What changes between two loads of a real store without its layout
// changing: related products in another order, a "viewing now" message with
// a new number, and reviews that load when scrolled into view.
(function () {
  var related = document.querySelector(".related-list");
  if (related) {
    var items = Array.prototype.slice.call(related.children);
    for (var index = items.length - 1; index > 0; index--) {
      var other = Math.floor(Math.random() * (index + 1));
      var swap = items[index];
      items[index] = items[other];
      items[other] = swap;
    }
    items.forEach(function (item) { related.appendChild(item); });
  }

  var message = document.querySelector(".viewing-now");
  if (message) {
    var count = 2 + Math.floor(Math.random() * 40);
    message.textContent = Math.random() < 0.5
      ? count + " people are looking at this right now"
      : "Popular, " + count + " sold today";
  }

  var reviews = document.querySelector(".reviews[data-lazy]");
  if (reviews && "IntersectionObserver" in window) {
    var observer = new IntersectionObserver(function (entries) {
      if (!entries.some(function (entry) { return entry.isIntersecting; })) return;
      observer.disconnect();
      reviews.innerHTML = reviews.querySelector("template").innerHTML;
    });
    observer.observe(reviews);
  }
})();
