namespace VisitorDemo;

public class VersionedContactsMetadata(int version) : IType<Contacts>
{
    public void Accept<V>(V visitor) where V : ITypeVisitor<Contacts>, allows ref struct
    {
        if (version >= 3)
            new ContactsMetadata(version).Accept(visitor);
        else
            StringMetadata.Instance.Accept(new AdapterVisitor<string, Contacts, V>(visitor, (ref Contacts c) => ref c.Email));
    }
}
