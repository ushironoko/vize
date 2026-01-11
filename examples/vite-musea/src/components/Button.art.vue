<script setup lang="ts">
import Button from "./Button.vue";
</script>

<art
  title="Button"
  component="./Button.vue"
  category="Components"
  status="ready"
>
  <variant name="Default" default>
    <Button>Default Button</Button>
  </variant>
  <variant name="Primary">
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Disabled Button</Button>
  </variant>
  <variant name="All Variants" skip-vrt>
    <div style="display: flex; gap: 1rem; flex-wrap: wrap">
      <Button>Default</Button>
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button disabled>Disabled</Button>
    </div>
  </variant>
</art>
