extends: default.tpl
---
<div class="post-list">
  <h2>This Week in Amethyst</h2>
  <br />
  {% for post in posts %}
   <a href="{{post.path}}">{{ post.title }}</a>
  {% endfor %}
</div>