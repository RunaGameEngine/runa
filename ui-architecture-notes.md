# UI Architecture — заметки по улучшению

## 1. Проблема текущего API

Сейчас каждое создание узла требует явно передавать `parent`:

```rust
let panel = ui_renderer.vbox(ui_renderer.root()).with_size(300, 200).id();
ui_renderer.text(panel, "Hello").with_font_size(24);
ui_renderer.text(panel, "World");
```

Это многословно и неинтуитивно — parent таскается руками.

## 2. Parent stack — главное улучшение

Идея: `UiNodeBuilder` хранит текущий родительский узел. Каждый `container/vbox/hbox` автоматически становится новым текущим родителем для последующих детей. `.end()` — возврат к предыдущему родителю.

Схема:
```
builder.vbox(...)       // новый parent (внутри стека)
  .text(...)            // ребёнок vbox
  .text(...)            // ребёнок vbox
  .hbox(...)            // новый parent внутри vbox
    .text(...)          // ребёнок hbox
    .image(...)         // ребёнок hbox
  .end()                // возврат к vbox
  .text(...)            // ребёнок vbox
.end()                  // возврат к корню
```

Альтернатива — closure-scope:
```rust
ui.vbox(|v| {
    v.text("Hello").font_size(24);
    v.button("Click", || action());
    v.hbox(|h| {
        h.image(&tex);
        h.text("caption");
    });
});
```

Closure-вариант чище (вложенность видна сразу, нет отслеживания `.end()`).

## 3. Новые виджеты

| Виджет | Тип узла | Нужен |
|--------|----------|-------|
| Container | уже есть | |
| HBox / VBox | уже есть | |
| Text | уже есть | |
| Image | уже есть | |
| Button | уже есть (container + text + клик) | |
| **Slider** | новый `UiNodeKind::Slider { value, min, max }` | mouse-drag |
| **Checkbox** | container + indicator + text + клик | |
| **Input** | container + text (редактируемый) + keyboard | |
| **ScrollContainer** | container с clip + смещением | |

## 4. Interaction — уйти от колбэков к event-driven

Сейчас `on_click` висит мёртвым грузом (передаётся в builder, но не хранится и не обрабатывается).

Лучше:
- Ввести `InteractionState` для каждого узла: `None / Hovered / Pressed / Dragging`
- На этапе `layout()` или отдельным проходом проверять позицию мыши против `computed.rect`
- Использовать `CursorInteractable` из `runa_core::systems::interaction_system` — там уже есть рейкаст мыши по объектам с коллайдерами
- Для UI проще сделать отдельный hit-test по дереву узлов (по Z-index sorted)

Изменения:
- `UiNode.interaction: InteractionState`
- `UiNode.interaction_callback: Option<Box<dyn FnMut(InteractionState)>>`
- В `build_render_commands` заодно проверять hover/press и дёргать колбэк

## 5. Immediate vs Retained

Твой движок **retained** — дерево узлов живёт между кадрами, мутируется по необходимости. Это правильно для игры:

- **+** Производительность: не пересоздаётся каждый кадр
- **+** Состояние UI не теряется
- **+** Легко интегрировать с ECS (`UiRenderer` как компонент)
- **–** Многословнее чем egui

Не пытайся переписать в immediate-mode. Retained — осознанный выбор для игрового движка.

## 6. StyleSheet — отделить стили от логики

Сейчас стили (background, font_size, tint) задаются через builder `.with_*` прямо при создании. Это смешивает структуру и оформление.

Можно ввести `StyleSheet`:
```rust
let button_style = StyleSheet::default()
    .with_background(0.2, 0.2, 0.3, 1.0)
    .with_background_hover(0.3, 0.3, 0.5, 1.0)
    .with_padding(10, 5, 10, 5);

ui.button("OK")
    .with_style_sheet(&button_style);
```

Хранить `style` как `StyleProps` уже есть в `UiNode`, но нет наследования тем.

## 7. Что делать сейчас — приоритеты

1. **Parent stack** (closures или `.end()`) — самый большой выигрыш по удобству
2. **Slider** как `UiNodeKind` + mouse-drag
3. **Interaction** — hover подсветка, рабочий клик
4. **Hit-test UI** — проверка мыши против узлов (по Z-index)
5. **StyleSheet** — вынести стили в отдельный слой
