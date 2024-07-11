<script>
  export let title = "Empty title";
  export let desc = "Empty description";
  export let preview = null;
  export let article_url = null;
</script>

<section>

<div class="media lead row justify-content-between">
  <div class="col-xs-8 col-md-4">
    <a class="pull-left" href={article_url}>
      <img class="media-object" data-src="holder.js/64x64" alt="64x64"
        src="{preview}"
        style="width: 8em; height: 8em; ">
      <br>
    </a>
  </div>
  <div class="media-body col-xs-8 col-md-8">
    <div class="article-preview-text">
      <h2 class="article-preview-header"><a href={article_url}>{title}</a></h2>
      <hr><br>
      {desc}
    </div>
  </div>
</div>
</section>
