# BLISS/11

Implementation of the BLISS compiler as described in "The Design of an Optimizing Compiler" (1973)
by Wulf, Johnson, Weinstock, and Hobbs.

I've owned the book for some years and only recently I realized that at least a couple of the original
authors are deceased, (Bill Wulf only recently), so I decided to go ahead and write an implementation
of the compiler, as a posthumous homage for their pioneering work.

The implementation is in Rust and won't follow the book exactly, as I'll be relying on data structures
that fit the language better.

Note that there are implementations of the language targetting modern architechtures, for OpenVMS, and
I'm aware of at least one [open-source implementation](https://github.com/madisongh/blissc), written in
C and targetting LLVM. The present work is simply an exercise on compiler construction, intended to be a
clean-room implementation using only pre-80s sources: the papers and books describing the languague and
the original compiler.

## General characteristics of the language

**Typeless:** Its only data type is the "word". It's up to the programmer to interpret a word in terms of integers, pointers, characters, etc.

**Case Insensitive:** As usual in languages of the era, identifiers and keywords are case-insensitive, so one can write them in capital or lowercase and it won't matter to the compiler. The first language to be sensitive to the input case, AFAIK, was C (ALGOL 60 had this too, at times, but it was implementation-dependent).

**Expression Oriented:** Instead of statements, Bliss constructs are expressions, even control flow ones.

**Explicit dereferencing:** An identifier name *always* indicates the address to a variable, not its value. To get the value the variable needs to be dereferenced.

**No GOTO statement:** Bliss was influenced by ALGOL and generally by the structured programming philosophy, and omits `GOTO`

**Highly optimizing compiler:** Back in the day the original CMU BLISS compiler was revolutionary for its very aggressive optimization techniques.

## Language Description

The description is according to [Wul71][^1].

### Storage

BLISS calls data storage "segments". A storage segment is made of a finite (and fixed) number of "words", which in turn are composed of a fixed (and finite) number of bits. In the original PDP-11 it was 36 bits, this will obviously be different for my implementation. Words can have contiguous sets of bits grouped into "fields". These fields can be "named", and the value of a name is called a "pointer" to that field. A whole words on itself is a field, and may be named. A BLISS program does not make distintitions over the contents of a field: they're just bits.

Examples of segment declarations:

    GLOBAL g;
    OWN x, y[5], z;
    LOCAL p[100];
    REGISTER r1, r2[3];
    FUNCTION f(a, b) = .a XOR .b;

The function declaration initializes a segment named `f` to the code of the function.

The segments introduced by declarations have sizes that are defaulted (e.g. for `g`) or specified (`p[100]`). The identifiers are lexically local to the block where they're declared, except for those declared **global**. Segments declared **global**, **own**, or **function** are created only once and preserved for the whole duration of the program. On the other hand **local** and **register** lifetime is restricted to block where they're declared. Additional declarations are **external** (refers to a **global** from a different module) and **bind**, which binds the result of an expression to an identifier:

    BIND y2 = y + 2, pa = p + a;

Names are bound to identifiers and, as mentioned in the previous section, their value is a pointer to the segment they represent. The dereferencing operator is the dot (`.`).

### Control

Every executable construct is an expression and computes a value. Expressions concatenated with semicolons (`;`) form a "compound expression" that has the value of the last of those expressions. One can use the pairs `BEGIN` and `END`, or opening/closing parentheses (see the grammar below) to enclose a compound expression and turn it into a simple expression. This is called a **block** and can include declarations.

The operator `=` can be read as "store into": `a = b` means "the bit pattern resulting from the evaluation of the expression `b` is to be stored in the field named by the pointer resulting from the evaluation of `a`. So, the C statement `x = x + 1;` would translate into BLISS as `x = .x + 1`. The usual binary and unary operators (arithmetic, logic) are present in the language. Logic operators return `1` if the relation is satisfied, `0` otherwise.

There are six different forms of control flow expressions:
* Conditional
* Looping
* Case-select
* Function call
* Coroutine call
* Escape
#### Conditional
Of the form `if e1 then e2 else e3`. Evaluates and has the value of the expression from the taken branch. The abbreviated form `if e1 then e2` is the same has having an implicit `else 0`.
#### Looping
There are six variants of looping expressions:

    WHILE e1 DO e
    DO e WHILE e1
    UNTIL e1 DO e
    DO e UNTIL e1
    INCR <name> FROM e1 TO e2 BY e3 DO e
    DECR <name> FROM e1 TO e2 BY e3 DO e

As the name implies, `while` and `until` imply negated conditions (something happens _while an expression evaluates true_, vs  _until an expression evaluates true_). The `do .. XXX ...` forms work almost the same, except that `e` will evaluate _at least once_, where as the variants `XXX ... do ...` might not evaluate at all.

The `incr` and `decr` forms are our familiar `for` loops from other languages. In both of them, the `by e3` syntax is optional. 

> [!NOTE]
> These expressions are not minimal. A `while .. do` would be enough to build all of the cases, but syntactic sugar is not a modern idea by any means.

> [!IMPORTANT]
> **The value of a loop expression is uniformly taken to be -1, except in the case of a escape expression within `e`**.

#### Case/Select

BLISS offers two _switch_-like expressions, but they work in a different way.

    CASE e OF SET e0; e1; e2; ..; en TES
    SELECT e OF NSET e0: e1; e2: e3; ..; e2n: e2n+1 TESN

The `e` for the `case` expression evaluates to an _index_ used to select one if the `ei` (0 <= `i` < `n`) expressions, which will be evaluated, becoming the value for the whole expression. If `e` evaluates to an index outside the valid range, the return value is undefined. The `select` expression is similar, but `e` is not restringed in range. Instead, `select` works as follows:

1. `e` is evaluated
2. For each `e2i` expression (0 <= `i` < `n`), the expression is evaluated and if `e == e2i`, then the expression `e2i+1`.

If no `e2i` matches `e`, the whole expression gets a value of -1. If more than one expression matches `e`, the value of `e2i+1` for the last matching `e2i` is taken as the value for the whole expression. Note that the expressions are matched in ascending order of `i`.

A `nil` value might have been a better choice for the undefined values, but there was no such available value in the PDP-11 and the designers decided for -1 as the lesser evil, because testing the sign of a value in PDP-11 was relatively cheap.

> [!NOTE]
> Same as with the looping expressions, both `case` and `select` are just convenient syntactic sugar for conditional expressions. The mean reason to include a rich set of control structures come in part from the design decision of completely excluding an arbitrary `goto` from the language.

#### Function calls

Function calls are of the form `e(e1, e2, .., en)`. This will cause activation of the subprogram named by `e`. Only call-by-value parameters are allowed, but call-by-references is available anyway given that pointer values are readily available in the language. The resulting value of a function is the one obtained from the execution of its body.

> [!NOTE]
> There's no need to explicitly name a function by its identifier: it's enough that `e` evaluates to the name of the segment containing the function code. So, for example, this is a valid function call:

    (CASE .x OF SET p1; p2; p3 TES)(.z)

In our case `p1`, `p2`, and `p3` could be for example function identifiers. They wouldn't be executed at that point, just selected and evaluated, resulting in the name of the segment containing their code.

#### Coroutines

The body of any function may be activated as a coroutine/async process. Each activation, whether as a subroutine or coroutine is independent of the others and arbitrarily many can coexist at a given time.
There are two primitives associated to coroutines:

    CREATE e(e^1, e^2, ..., e^n) AT e2 LENGTH e3 THEN e4
    EXCHJ (e5, e6)

`CREATE` establishes a new context (AKA, a stack) for the function named in `e`. The stack is set up beginning at word `e2` and its size will be of `e3` words. The activation point is the head of the function named by `e`. At the point of `CREATE` the coroutine will not be activated. `e4` won't be evaluated either. The value for the `CREATE` expression will be the "process name" for a new coroutine.

`EXCHJ` allows to switch from the currently executing context and a different one by performing an "exchange jump". Control will be passed to the coroutine named by `e5`, and `e6` will become the value of the `EXCHJ` expression that was last used to transfer control out of the coroutine named by `e5`. For example:

    BEGIN
      OWN pa, pb, s1[100], s2[100];
      a = FUNCTION() =
        BEGIN
           ...
           x = EXCHJ(.pb, 1);
           ...
        END;
      b = FUNCTION() =
        BEGIN
           ...
           y = EXCHJ(.pa, 2);
           ...
        END;
      pa = CREATE a() AT s1 LENGTH 100 THEN exit;
      pb = CREATE b() AT s2 LENGTH 100 THEN exit;
      EXCHJ(.pa, 0);
    END

Two functions (`a`, `b`) are declared, and then two coroutines are created (names bound to `pa` and `pb`). Finally, we start the corouting named at `pa`, passing `0` as a value. No `EXCHJ` has been used to leave the coroutine at `pa` at this point, so the `0` is ignored. Eventually, `x = EXCHJ(.pb, 1);` is reached, passing control to the coroutine at `pb`, which starts execution similarly to `pa`.
Once the control flow reaches `y = EXCHJ(.pa, 2);`, control passes back to the first coroutine. But this time, control had left earlier
by means of an `EXCHJ`, meaning that the value passed (`2`) becomes the return value of that original expression, and gets stored as
`.x`.

We haven't talked yet about the expression `e4`: it's executed only when and if control passes out of the _body_ of the coroutine by a normal subroutine-type return. The normal (minimal) action expected of `e4` is returning the stack space used by the coroutine and `EXCHJ` to another, active coroutine.

#### Escape expressions

Given that `GOTO` was excluded from the language, a number of "escape" expressions were provided. There are 8 in total, targeting different control environments:

    EXITBLOCK e           EXITCASE e
    EXITCOMPOUND e        EXITSELECT e 
    EXITLOOP e            EXIT e
    EXITSET e             RETURN e

Each one of them exit from a specitic kind of structure (block, compound, loop, ...) Additionally, `EXIT` returns from any from of control expression, and `RETURN` does from function calls.

> [!NOTE]
> The decision to make BLISS a goto-less language and the decision of making it an "expression language" go hand in hand. A goto, for example, could allow one to get out of an expression without
> returning a value, something that is just not possible in BLISS.

### Data Structures

Earlier we've mentioned pointers, but keeping things relatively vague. A pointer in BLISS is a 5-tuple that consists of:
* A word address (_wa_)
* A field position (_p_)
* A field size (_s_)
* A register name
* An "indirect address" bit
All those get encoded (at least in the original language) in a single word and can be manipulated by the language. More
details can be found in the Reference Manual[^2]. In BLISS the notation to specify a pointer, using only the first 3
values, is _wa<p,s>_. Take for example:

    OWN x[100]

In the PDP-11, this declaration would bind `x` to a pointer to the 36-bit field which is the first word of a 100-word
segment. So, `wa` of the pointer `x` is the location allocated to the segment, `p` is 0, and `s` is 36 (bits). This
might be confusing (one could expect size to be 100), but the pointer is to the **first word** of the segment, which
is a 36-bit field, and we'd express it as _x<0, 36>. _X<3,4>_ would then be a pointer to a 4-bit field 3 bits from the
right end of a word named `X`. The word address was encoded in the lower bits of the pointer, so that pointer arithmetic
would work by words when adding small integers. So, if `x` is a pointer, then `x+1` points to word following that pointed
by `x`.

> [!IMPORTANT]
> The original PDP-11 computers could address up to 64KB (32K words) and later, higher end ones could address up to 4MB.
> Such addressing could be done with just 22 bits (and the words were 36 bits!), and even at that, this was achieved
> using an MMU, and individual processes were typically limited to a virtual address space of just 64KB (16-bit addressing),
> meaning that a large chunk of a pointer was available for encoding the other attributes.
>
> A 64-bit register as available in modern X86_64 or AArch64 can address enormous amounts of virtual RAM, but this is of course
> limited in practice by the amount of physical RAM available in the system (e.g., 16-32GB are the most common amounts seen
> in laptops or desktops at the time of writing, and larger than this is typically seen only in very high end workstations
> or servers, particularly now that AI can benefit of unified memory schemes). Even 128GB can be addressed by a mere 37 bits,
> meaning that there's ample room to describe the rest of the attributes.
>
> Even furhter, the OS will usually limit the amount of RAM available to each process. In Linux (the target for this compiler),
> while it's possible to use up to 52 or 57 bits (architecture dependent) for the address, it's typically to limit it by
> default to 48 bits for compatibility reasons. And this would allow addressing terabytes of RAM, which is way, way beyond
> practical.

> [!NOTE]
> Given the limitations mentioned above, there's plenty of room in the upper bits of our pointers to encode the additional
> information. This might be a bit more challenging if targeting a 32-bit architecture, because addresing the maximum 4GB
> has been a realistic possibility for many years, but I'm not going target any such architecture at this time. I'll cross
> that particular bridge when (and if) I get to it.

#### Mapping

Back in the day, the designers of BLISS looked at how systems programmers worked and noticed that a lot of their design
effort was spent in figuring out how to encode and access the information they were working on, efficiently. This was
a big design driver for the language. They wanted two things:
* The user must be able to specify an _accessing algorithm_ for the structure's individual elements.
* The structure definition and the algorithms must be separate - an early jab at separation of concerns, in the hope
  that modifications on one side of the equation wouldn't have (much of) an effect on the other.
With this in mind they derived their pointer design (see above) and introduced a couple of primitives that allowed
describing structure mechanisms:

    STRUCTURE <name>[<formal parameter list>] = e
    MAP <name> <name list>

The former allows the definition of a class of structures by suplying an algorithm accessing the structure elements.
The latter allows associating a structure class to a colon-separated list of names. Example:

    BEGIN
      STRUCTURE ary2[i, j] = (.ary2 + .i * 10 + .j);
      OWN x[100], y[100], z[100];
      MAP ary2 x:y:z;
      ...
      x[.a, .b] = .y[.b, .a];
      ...
    END

This piece of code declares a structure `ary2` to address two-dimensional 10xn arrays. Then it proceeds to declare three
segments of 100 words each, which finally get associated to the structure class `ary2` using the `MAP` declaration. By
doing that the `x[e1, e2]` and `y[e3, e4]` (and `z[e5, e6]`, but it's not shown here) forms are valid
**within this block**. The `STRUCTURE` declaration then would work as a sort of macro. The example expression is probably
transposing the contents of `y` into `x`.

In a way, the way the expression defined with `STRUCTURE` functions is similar to that of an object's method. We could
do something equivalent by writing:

    FUNCTION ary2(f0, f1, f2) = (.f0 + .f1 * 10 + .f2)
    OWN x[100], y[100], z[100];
    ...
    ary2(x, .a, .b) = ary2(y, .b, .a)
    ...

where `f0` would take the same role as an explicit `this` or `self` in object-oriented languages. The main practical
difference between the above `FUNCTION` and `STRUCTURE` definitions, besides the implied "argument", is that the
function allocates storage in the stack for its formal parameters, as common for any function call that we're used
to, but the code generated when mapping a structure doesn't seem to do this, and of course there's no branching to
and returning from the function. It would be more akin to an explicit function inlining.

<TBD: More on parameterized structures>

## Tentative grammar

    program             = expression ;
    
    expression          = assignment_expr 
                        | control_expr 
                        | simple_expr
                        ;
    
    assignment_expr     = primary , "=" , expression ;
    
    simple_expr         = term , { binary_operator , term } ;
    
    term                = [ unary_operator ] , primary ;
    
    primary             = identifier           (* Evaluates to the memory ADDRESS *)
                        | "." , primary        (* Explicit Dereference: fetches VALUE *)
                        | integer_constant
                        | string_literal
                        | routine_call
                        | "BEGIN" , expression , "END"
                        | "(" , expression , ")"
                        | block
                        ;
    
    binary_operator     = "+" | "-" | "*" | "/"
                        | "MOD" | "AND" | "OR" | "XOR" | "EQV"
                        | "EQL" | "NEQ" | "LSS" | "LEQ" | "GTR" | "GEQ"
                        ;
    
    unary_operator      = "-"
                        | "NOT"
                        ;
    
    block               = "BEGIN" , { declaration } , expression_elements , "END"
                        | "(" , { declaration } , expression_elements , ")"
                        ;
    
    expression_elements = expression , { ";" , expression } ;
    
    declaration         = storage_declaration 
                        | bind_declaration 
                        | routine_declaration 
                        | structure_declaration
                        | map_declaration
                        ;
    
    storage_declaration = ( "OWN" | "LOCAL" | "GLOBAL" | "EXTERNAL" | "REGISTER" ) , allocation_list , ";" ;
    allocation_list     = allocation_item , { "," , allocation_item } ;
    allocation_item     = identifier , [ "[" , integer_constant , "]" ] ;
    
    bind_declaration    = "BIND" , bind_list , ";" ;
    bind_list           = bind_item , { "," , bind_item } ;
    bind_item           = identifier , "=" , expression ;

    (* STRUCTURE defines the access algorithm formula *)
    structure_declaration = "STRUCTURE" , structure_list , ";" ;
    structure_list        = structure_item , { "," , structure_item } ;
    structure_item        = identifier , [ "[" , formal_parameters , "]" ] , "=" , expression ;
    formal_struct_params  = identifier , { ", " , identifier } ;

    (* MAP binds a structure formula to actual memory blocks *)
    map_declaration       = "MAP" , identifier , map_list , ";" ;
    map_list              = identifier , { ":" , identifier } ;
    
    control_expr       = if_expr 
                       | conditional_loop 
                       | step_loop 
                       | case_expr
                       | seelct_expr
                       | leave_expr 
                       | return_expr
                       | coroutine_expr
                       ;
    coroutine_expr     = create_expr 
                       | exchj_expr ;
    
    if_expr            = "IF" , expression , "THEN" , expression , [ "ELSE" , expression ] ;
    
    conditional_loop   = ( "WHILE" | "UNTIL" ) , expression , "DO" , expression
                       | "DO" , expression , ( "WHILE" | "UNTIL" ) , expression
                       ;
    
    (* INCR/DECR handle structured iteration across memory/counters *)
    step_loop          = ( "INCR" | "DECR" ) , identifier , "FROM" , expression , "TO" , expression , [ "BY" , expression ] , "DO" , expression ;

    case_expr          = "CASE" , expression , "FROM" , "0" , "TO" , integer_constant , "OF" , 
                         "SET" , 
                         expression_elements , 
                         "TES" ;
    
    select_expr        = "SELECT" , expression , "OF" , 
                         "NSET" , 
                         { select_component } , 
                         "TESN" ;

    select_component   = select_label , ":" , expression , ";" ;

    (* OTHERWISE was not present in the original paper. It was added during            *)
    (* implementation to prevent bugs due to undefined behavior                        *)
    select_label       = expression       (* Matches if .selector_expr EQL .label_expr *)
                       | "OTHERWISE" ;    (* Default fallback branch *)
    
    (* LEAVE exits a named block prematurely with a specific value *)
    leave_expr         = "LEAVE" , identifier , [ "WITH" , expression ] ;
    
    return_expr        = "RETURN" , [ expression ] ;
    
    routine_declaration = [ "GLOBAL" ] , "ROUTINE" , identifier , [ "(" , formal_parameters , ")" ] , "=" , expression , ";" ;
    
    formal_parameters   = identifier , { "," , identifier } ;
    
    routine_call        = identifier , "(" , [ actual_parameters ] , ")" ;
    
    actual_parameters   = expression , { "," , expression } ;
    
    identifier         = letter , { letter | digit | "_" } ;

    (* CREATE instantiates the coroutine environment *)
    create_expr        = "CREATE" , identifier , "(" , actual_parameters , ")" , 
                         "AT" , expression , 
                         "LENGTH" , expression , 
                         "THEN" , expression ;

    (* EXCHJ yields/swaps execution to another coroutine context *)
    exchj_expr         = "EXCHJ" , "(" , expression , "," , expression , ")" ;
    
    integer_constant   = decimal_digits
                       | binary_digits
                       | octal_digits
                       | hexadecimal_digits
                       ;
    
    decimal_digits     = digit , { digit } ;
    binary_digits      = '%', 'B', binary_digit , { binary_digit } ;
    octal_digits       = '%', 'O', octal_digit , { octal_digit } ;
    hexadecimal_digits = '%', 'X', hexadecimal_digit , { hexadecimal_digit } ;
    
    string_literal     = "'" , { character - "'" | "''" } , "'" ;
    
    letter             = "A" | ... | "Z" ;
    binary_digit       = "0" | "1" ;
    octal_digit        = "0" | ... | "7" ;
    digit              = "0" | ... | "9" ;
    hexadecimal_digit  = "0" | ... | "9" | "A" | ... | "F" ;


## Literature

[^1]: **[Wul71]** W. A. Wulf, D. B. Russell, and A. N. Habermann "BLISS: A Language for System Programming," CACM 14,12 (Dec. 1971), 780-790
[^2]: **[Wul72]** W. A. Wulf et al, "BLISS reference manual", Computer Science Dept. Rep. Carnegie-Mellon University.
