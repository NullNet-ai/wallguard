#include <platform/revision.h>

#include <libxml/parser.h>
#include <libxml/tree.h>

#include <string.h>

STATIC_ASSERT(sizeof(((revision *)0)->username) == USERNAME_LENGTH,
              "Username Is Expected To Be An Array Of Correct Size");

static boolean_t parse_opnsense_revision(revision *rev) {
    static const char filename[] = "/conf/config.xml";

    xmlDoc *document = xmlReadFile(filename, NULL, 0);
    if (!document) {
        return WM_FALSE;
    }

    xmlNode *root = xmlDocGetRootElement(document);
    if (!root) {
        xmlFreeDoc(document);
        return WM_FALSE;
    }

    xmlNode *currentNode = NULL;
    for (currentNode = root->children; currentNode; currentNode = currentNode->next) {
        if (currentNode->type == XML_ELEMENT_NODE && strcmp((const char *)currentNode->name, "revision") == 0) {
            xmlNode *revisionChild = NULL;
            for (revisionChild = currentNode->children; revisionChild; revisionChild = revisionChild->next) {
                if (revisionChild->type == XML_ELEMENT_NODE) {
                    if (strcmp((const char *)revisionChild->name, "time") == 0) {
                        char *time_str = (char *)xmlNodeGetContent(revisionChild);

                        rev->time = (time_t)atof(time_str);

                        xmlFree(time_str);
                    } else if (strcmp((const char *)revisionChild->name, "username") == 0) {
                        char *username = (char *)xmlNodeGetContent(revisionChild);

                        memset(rev->username, 0, sizeof(rev->username));
                        strncpy(rev->username, username, sizeof(rev->username) - 1);

                        xmlFree(username);
                    }
                }
            }
        }
    }

    xmlFreeDoc(document);
    xmlCleanupParser();

    return WM_TRUE;
}

static boolean_t parse_pfsense_revision(revision *rev) {
    // Since pfSense and OPNsense have similar `config.xml` structures,
    // we can use the OPNsense version, as it is more general.
    // OPNsense reports timestamps as floating-point values, while pfSense does not.
    // Therefore, the OPNsense version of the parsing function should be capable of
    // parsing both integral and floating-point timestamps, which it is.
    return parse_opnsense_revision(rev);
}

boolean_t obtain_revision(platform_type platform, revision *rev) {
    if (rev == NULL) {
        return WM_FALSE;
    }

    if (platform == PLATFORM_OPNSENSE) {
        return parse_opnsense_revision(rev);
    } else if (platform == PLATFORM_PFSENSE) {
        return parse_pfsense_revision(rev);
    }

    return WM_FALSE;
}
