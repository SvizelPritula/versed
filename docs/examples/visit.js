const User = s => s.s/*truct*/(
    "name", s => s.t/*ext*/(),
    "age", s => s.v/*ersion*/ > 10 ? s.n/*umber*/() : s.u/*nit*/()
);

// Recursive types
const A = s => s.s/*truct*/("value", B);
const B = s => s.e/*num*/(A, s => s.u/*nit*/());
