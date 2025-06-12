namespace VisitorDemo;

public class VersionedContactsMetadata(int version) : IType<Contacts>
{
    public void Accept(ITypeVisitor<Contacts> visitor)
    {
        if (version >= 3)
            new ContactsMetadata(version).Accept(visitor);
        else
            StringMetadata.Instance.Accept(new AdapterVisitor<string, Contacts>(visitor, c => c.Email));
    }
}
