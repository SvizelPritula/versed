version v1;

User = struct {
    name: string,
    age: enum { known: int, unknown },
    contacts: [Contact],
};

Contact = enum {
    phone: int,
    email: string,
    address: struct {
        street: string,
        city: string,
        country: string,
    },
};
