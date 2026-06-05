# 网页源 markdown 文档写法规范

## I. 列表

### 1.1 列表

列表开始前后必须是空行，例如可以是：

```markdown
列表开始：

- item1
- item2

列表结束
```

但是不能是：

```markdown
列表开始：
- item1
- item2
列表结束
```

### 1.2 嵌套列表 

嵌套列表的缩进层级为 4 个空格，例如

```markdown
- item1
    - item1.1
    - item1.2
- item2
    - item2.1
```

不能是 2, 3 个空格，这些都无法正常被 Mkdocs 编译。

### 1.3 列表内嵌引用

这种写法一般不常用，但如果需要在列表项内嵌引用，推荐的写法是：

列表叠加引用的写法如下：

**引用缩进四个空格，前后空行（推荐这种方式）**

```markdown
1. item1

    > 这是一段列表项内部的引用。

2. item2
    
    > 内部引用

3. item3
```

或者引用**缩进三个空格，前后不用空行（不建议使用这种方式，mmf 工具不支持这种格式）**：

```markdown
1. item1
   > 这是一段列表项内部的引用。
2. item2
   > 内部引用
3. item3
```

### 1.4 列表内嵌代码块

这种写法一般不常用，但如果需要在列表项内嵌代码块，推荐的写法是：

**代码块需要缩进四个空格，并且代码块前后需要空行**

````markdown
- item1

    ```python
    print("inside list item")
    print("inside list item")
    print("inside list item")
    ```

- item2
````

不能是：

````markdown
- item1
    ```python
    print("inside list item")
    print("inside list item")
    print("inside list item")
    ```
- item2
````

也不能是：

````markdown
- item1

   ```python
   print("inside list item")
   print("inside list item")
   print("inside list item")
   ```

- item2
````

## II. 公式

### 2.1 行间公式

**行间公式必须前后空行**（否则会被渲染为行内公式），例如：

```markdown
这是行间公式：

$$E = mc^2$$

这是行间公式：

$$
F = ma
$$

结束
```

不能是：

```markdown
这是行间公式：
$$E = mc^2$$
这是行间公式：
$$
F = ma
$$
结束
```

### 2.2 行内公式

行内公式无特殊要求

## III. 代码块

代码块整体前后必须空行，例如：

```markdown
这是代码块：

```python
print("hello, world!")
print("hello, world!")
```

结束
```

不能是：

```markdown
这是代码块：
```python
print("hello, world!")
结束
```

## IV. 表格

表格前后必须空行，例如：

```markdown
这是表格：

| Name | Age |
| --- | --- |
| Alice | 30 |

结束
```

不能是：

```markdown
这是表格：
| Name | Age |
| --- | --- |
| Alice | 30 |
结束
```


