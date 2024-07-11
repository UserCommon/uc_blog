<script lang="ts">
  import {
    Breadcrumb,
    Paginator,
    Search,
    ArticlePrev
  } from "$lib/articles"

  function getArticle(title: string, description: string) {
    return {title: title, desc: description};
  }

  const articles = [
    getArticle("Wow", "And wow"),
    getArticle("No", "Yep")
  ];
</script>


<section>
  <div class="block-container ext-container">
    <div class="row">
      <div class="col-xs-8 col-md-8">
        <Breadcrumb />
        <article>
        {#each articles as article}
          <ArticlePrev
            title = {article.title}
            desc  = {article.desc}
          />
        {/each}
        </article>
      </div>
      <div class="col-xs-8 col-md-4">
        <Search />
      </div>

    </div>
  </div>
  
  <Paginator />
</section>
