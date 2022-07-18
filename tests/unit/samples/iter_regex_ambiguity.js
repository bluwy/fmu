// Regex is surprisingly annoying to parse because of division.
// Credit: https://github.com/guybedford/es-module-lexer/blob/559a550318fcdfe20c60cb322c147905b5aadf9f/test/_unit.cjs#L510
/as)df/; x();
a / 2; '  /  '
while (true)
  /test'/
x-/a'/g
try {}
finally{}/a'/g
(x);{f()}/d'export { b }/g
;{}/e'/g;
{}/f'/g
a / 'b' / c;
/a'/ - /b'/;
+{} /g -'/g'
('a')/h -'/g'
if //x
('a')/i'/g;
/asdf/ / /as'df/; // '
p = `\${/test/ + 5}`;
/regex/ / x;
function m() {
  return /*asdf8*// 5/;
}
export { a };